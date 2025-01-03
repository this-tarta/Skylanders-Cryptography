#![allow(unused)]

use crc;
use aes::Aes128;
use block_modes::{block_padding::ZeroPadding, BlockMode};
use block_modes::Ecb;
use md_5::{Md5, Digest};
use std::cmp::min;
use std::{u16, u32};
use std::{fs::{self, File}, io::{self, Read, Seek, Write}, path::Path};


use crate::skyfigures::Character;
use crate::skyvariants::Variant;
use crate::skyhats::Hat;

type Aes128Ecb = Ecb<Aes128, ZeroPadding>;

const BLOCK_SIZE: usize = 16;
const BLOCKS_PER_SECTOR: usize = 4;
const NUM_SECTORS: usize = 16;
/// The number of bytes that a Skylander figure (Mifare 1K NFC tag) stores
const NUM_BYTES: usize = BLOCK_SIZE * BLOCKS_PER_SECTOR * NUM_SECTORS;

/// AREA_BOUNDS[i] is the bounds [start, end) of area i
static AREA_BOUNDS: [(usize, usize); 4] = [(0x80, 0x110), (0x240, 0x2D0), (0x110, 0x160), (0x2D0, 0x320)];

static HASH_CONST: &[u8] = &[
    0x20, 0x43, 0x6F, 0x70, 0x79, 0x72, 0x69, 0x67, 0x68, 0x74, 0x20, 0x28, 0x43, 0x29, 0x20, 0x32,
    0x30, 0x31, 0x30, 0x20, 0x41, 0x63, 0x74, 0x69, 0x76, 0x69, 0x73, 0x69, 0x6F, 0x6E, 0x2E, 0x20,
    0x41, 0x6C, 0x6C, 0x20, 0x52, 0x69, 0x67, 0x68, 0x74, 0x73, 0x20, 0x52, 0x65, 0x73, 0x65, 0x72,
    0x76, 0x65, 0x64, 0x2E, 0x20
];
static CRC16_CCITT_FALSE: crc::Algorithm<u16> = crc::Algorithm {
    width: 16,
    poly: 0x1021,
    init: 0xFFFF,
    refin: false,
    refout: false,
    xorout: 0,
    check: 0x29B1,
    residue: 0
};

/// Calculates key_a based on the game's unique CRC-48 checksum
fn key_a(bytes: &[u8]) -> u64 {
    const CUSTOM_ALG: crc::Algorithm<u64> = crc::Algorithm {
        width: 48,
        poly: 0x42f0e1eba9ea3693,
        init: 2 * 2 * 3 * 1103 * 12868356821,
        refin: false,
        refout: false,
        xorout: 0,
        check: 0,
        residue: 0,
    };

    let crc = crc::Crc::<u64>::new(&CUSTOM_ALG);

    crc.checksum(bytes)
}

pub struct Skylander {
    data: Box<[u8; NUM_BYTES]>
}

impl Skylander {
    
    /// Generates an empty Skylander based on the toy and variant provided
    /// (not guaranteed that character and variant match -- improper matches will lead to "special" tag in-game).
    /// Returns: figure data
    /// - figure will have an nuid that is provided (default is 00 00 00 00).
    /// - figure will be reset, but ownership is not taken automatically when used.
    pub fn new<T>(toy: T, variant: Variant, nuid: Option<[u8; 4]>) -> Self  where T: Into<u16>{
        let mut data = Box::new([0u8; NUM_BYTES]);
        let uid = match nuid {
            Some(v) => v,
            None => [0u8; 4]
        };
        let mut bcc = uid[0];
        for i in &uid[1..] {
            bcc ^= *i;
        }
    
        data[..4].copy_from_slice(&uid);
        data[4] = bcc;
    
        // SAK
        data[5] = 0x81;
        
        // ATQA
        data[6..=7].copy_from_slice(&[0x01, 0x0F]);
        debug_assert!(data[6] == 0x01 && data[7] == 0x0F);
    
        // Toy
        data[BLOCK_SIZE..=BLOCK_SIZE + 1].copy_from_slice(&toy.into().to_le_bytes());
    
        // Variant
        data[BLOCK_SIZE + 0xC..=BLOCK_SIZE + 0xD].copy_from_slice(&(variant as u16).to_le_bytes());
    
        let crc = crc::Crc::<u16>::new(&CRC16_CCITT_FALSE);
        let checksum = crc.checksum(&data[..0x1E]);
        data[0x1E..=0x1F].copy_from_slice(&u16::to_le_bytes(checksum));
    
        // Sector 0 trailer
        data[(3 * BLOCK_SIZE)..(3 * BLOCK_SIZE + 10)]
            .copy_from_slice(&[0x4B, 0x0B, 0x20, 0x10, 0x7C, 0xCB, 0x0F, 0x0F, 0x0F, 0x69]);
    

        calculate_key_a(&mut data);
        write_ones(&mut data);
        
        Self { data }
    }

    /// Saves the Skylander to a file
    /// Overwrites any data up to 1KB from seek start
    fn save_to_file(&self, file: &mut File) -> io::Result<()> {
        let mut data = *(self.data).clone();
        calculate_checksums(&mut data);
        encryption_skylander(&mut data, true);
        file.seek(io::SeekFrom::Start(0))?;
        file.write_all(&data)?;

        Ok(())
    }

    /// Saves the Skylander to a file
    /// Overwrites any existing data
    pub fn save_to_filename(&self, filename: &str) -> io::Result<()> {
        self.save_to_filepath(Path::new(filename))
    }

    /// Saves the Skylander to a file
    /// Overwrites any existing data
    pub fn save_to_filepath(&self, path: &Path) -> io::Result<()> {
        if path.exists() {
            let tmp_path = match path.parent() {
                Some(p) => p.join("tmp.sky"),
                None => return Err(io::Error::other("can't save to root dir"))
            };
            let mut tmp = File::create(&tmp_path)?;
            self.save_to_file(&mut tmp)?;

            fs::rename(&tmp_path, path)
        } else {
            let mut file = File::create_new(path)?;
            self.save_to_file(&mut file)
        }
    }

    /// Reads a Skylander from a file
    /// Does not verify the data integrity of the file
    pub fn from_filepath(path: &Path) -> io::Result<Self> {
        let mut data = Box::new([0u8; NUM_BYTES]);
        let mut file = File::open(path)?;
        file.read_exact(&mut *data)?;

        encryption_skylander(&mut *data, false);

        Ok(Self { data })
    }

    /// Reads a Skylander from a file
    /// Does not verify the data integrity of the file
    pub fn from_filename(filename: &str) -> io::Result<Self> {
        Self::from_filepath(Path::new(filename))
    }

    /// Sets gold of the Skylander to a specified value
    /// Note that in-game, the gold is capped at 65000
    pub fn set_gold(&mut self, gold: u16) {
        self.data[AREA_BOUNDS[0].0 + 0x3..= AREA_BOUNDS[0].0 + 0x4].copy_from_slice(&gold.to_le_bytes());
        self.data[AREA_BOUNDS[1].0 + 0x3..= AREA_BOUNDS[1].0 + 0x4].copy_from_slice(&gold.to_le_bytes());
    }

    /// Sets gold of Skylander to max
    pub fn max_gold(&mut self) {
        self.set_gold(u16::MAX);
    }
    
    /// Sets experience points of the Skylander to specified value
    /// Max experience in Spyro's Adventure is 33000 (level 10)
    ///                in Giants is 96500 (level 15)
    ///                in Swap Force and beyond is 197500 (level 20)
    /// This function will take min(xp, 197500)
    pub fn set_xp(&mut self, xp: u32) {
        let xp1 = min(xp, 33000);
        let xp2 = min(xp - xp1, 63500);
        let xp3 = min(xp - xp1 - xp2, 101000);

        debug_assert!(xp1 + xp2 + xp3 == xp || xp1 + xp2 + xp3 == 197500);

        let xp1_bytes = (xp1 as u16).to_le_bytes();
        let xp2_bytes = (xp2 as u16).to_le_bytes();
        let xp3_bytes = xp3.to_le_bytes();
        
        self.data[AREA_BOUNDS[0].0 ..= AREA_BOUNDS[0].0 + 0x1].copy_from_slice(&xp1_bytes);
        self.data[AREA_BOUNDS[1].0 ..= AREA_BOUNDS[1].0 + 0x1].copy_from_slice(&xp1_bytes);

        self.data[AREA_BOUNDS[2].0 + 0x3 ..= AREA_BOUNDS[2].0 + 0x4].copy_from_slice(&xp2_bytes);
        self.data[AREA_BOUNDS[3].0 + 0x3 ..= AREA_BOUNDS[3].0 + 0x4].copy_from_slice(&xp2_bytes);
        
        self.data[AREA_BOUNDS[2].0 + 0x8 .. AREA_BOUNDS[2].0 + 0xB].copy_from_slice(&xp3_bytes[..3]);
        self.data[AREA_BOUNDS[3].0 + 0x8 .. AREA_BOUNDS[3].0 + 0xB].copy_from_slice(&xp3_bytes[..3]);
    }

    /// Sets experience points of skylander to max
    pub fn max_xp(&mut self) {
        self.set_xp(u32::MAX);
    }

    /// Clears all data from the skylander
    pub fn clear(&mut self) {
        for i in 1..NUM_SECTORS {
            let sector_start = i * BLOCKS_PER_SECTOR * BLOCK_SIZE;
            let sector_trailer = sector_start + (BLOCKS_PER_SECTOR - 1) * BLOCK_SIZE;
            self.data[sector_start..sector_trailer].copy_from_slice(&[0u8; (BLOCKS_PER_SECTOR - 1) * BLOCK_SIZE]);
        }

        write_ones(&mut *self.data);
    }

    /// Sets the hat on the Skylander
    pub fn set_hat(&mut self, hat: Hat) {
        self.data[0x94..=0x95].copy_from_slice(&(hat as u16).to_le_bytes());
        self.data[0x254..=0x255].copy_from_slice(&(hat as u16).to_le_bytes());
    }
}

/// Encrypts or decrypts the 1K byte Skylander data. Does not check the validity of sector 0 data,
/// nor of the data blocks (checksums must be calculated before encryption and after decryption)
/// - if encrypt is true, then the data in Sectors 1 through 15 (excl. sector trailers) will be encrypted,
///   decrypted if false
/// writes in-place
pub fn encryption_skylander(data: &mut [u8; NUM_BYTES], encrypt: bool) {
    let mut seed = [0u8; 0x56];
    seed[..0x20].copy_from_slice(&data[..0x20]);
    seed[0x21..].copy_from_slice(HASH_CONST);

    for i in 1..NUM_SECTORS {       
        for j in 0..BLOCKS_PER_SECTOR - 1 {
            let block_idx = BLOCKS_PER_SECTOR * i + j;
            seed[0x20] = block_idx as u8;
            let hash = Md5::digest(&seed);

            let cipher = Aes128Ecb::new_from_slices(hash.as_slice(), Default::default()).expect("Iv error");

            // Because symmetric, this is the only difference between encryption and decryption
            if encrypt {
                cipher.encrypt(&mut data[block_idx * BLOCK_SIZE..(block_idx + 1) * BLOCK_SIZE], BLOCK_SIZE).expect("block mode error");
            } else {
                cipher.decrypt(&mut data[block_idx * BLOCK_SIZE..(block_idx + 1) * BLOCK_SIZE]).expect("block mode error");
            }
        }
    }
}

/// Calculates all checksums, writes in place to data array
pub fn calculate_checksums(data: &mut [u8; NUM_BYTES]) {
    // Checksum placeholders
    data[0x8E] = 0x05;
    data[0x8F] = 0x00;
    data[0x24E] = 0x05;
    data[0x24F] = 0x00;
    data[0x110] = 0x06;
    data[0x111] = 0x01;
    data[0x2D0] = 0x06;
    data[0x2D1] = 0x01;

    
    let crc = crc::Crc::<u16>::new(&CRC16_CCITT_FALSE);
    // Type 3
    let mut type_3_seed = [0u8; 0x110];  // has 0 padding; only 0x30 bytes are variable
    type_3_seed[..0x20].copy_from_slice(&data[0xD0..0xF0]);
    type_3_seed[0x20..0x30].copy_from_slice(&data[0x100..0x110]);

    data[0x8A..=0x8B].copy_from_slice(&crc.checksum(&type_3_seed).to_le_bytes()); // area 0
    
    type_3_seed[..0x20].copy_from_slice(&data[0x290..0x2B0]);
    type_3_seed[0x20..0x30].copy_from_slice(&data[0x2C0..0x2D0]);
    data[0x24A..=0x24B].copy_from_slice(&crc.checksum(&type_3_seed).to_le_bytes()); // area 1
    
    // Type 2
    let mut type_2_seed = [0u8; 0x30];
    type_2_seed[..0x20].copy_from_slice(&data[0x90..0xB0]);
    type_2_seed[0x20..].copy_from_slice(&data[0xC0..0xD0]);
    
    data[0x8C..=0x8D].copy_from_slice(&crc.checksum(&type_2_seed).to_le_bytes());  // area 0
    
    type_2_seed[..0x20].copy_from_slice(&data[0x250..0x270]);
    type_2_seed[0x20..].copy_from_slice(&data[0x280..0x290]);
    
    data[0x24C..=0x24D].copy_from_slice(&crc.checksum(&type_2_seed).to_le_bytes()); // area 1
    
    // Type 1
    let mut type_1_seed = [0u8; 0x10];
    type_1_seed.copy_from_slice(&data[0x80..0x90]);
    data[0x8E..=0x8F].copy_from_slice(&crc.checksum(&type_1_seed).to_le_bytes());  // area 0
    
    type_1_seed.copy_from_slice(&data[0x240..0x250]);
    data[0x24E..=0x24F].copy_from_slice(&crc.checksum(&type_1_seed).to_le_bytes());  // area 1

    // Type 6
    let mut type_6_seed = [0u8; 0x40];
    type_6_seed[..0x20].copy_from_slice(&data[0x110..0x130]);
    type_6_seed[0x20..].copy_from_slice(&data[0x140..0x160]);

    data[0x110..=0x111].copy_from_slice(&crc.checksum(&type_6_seed).to_le_bytes()); // area 2

    type_6_seed[..0x20].copy_from_slice(&data[0x2D0..0x2F0]);
    type_6_seed[0x20..].copy_from_slice(&data[0x300..0x320]);

    data[0x2D0..=0x2D1].copy_from_slice(&crc.checksum(&type_6_seed).to_le_bytes()); // area 3
}

/// Calculates and sets the key A for each sector trailer
fn calculate_key_a(data: &mut [u8; NUM_BYTES]) {
    // Sectors 1 through 15 trailers
    for i in 1..NUM_SECTORS {
        let mut bytes = [0u8; 5];
        let curr_block = (i * BLOCKS_PER_SECTOR + (BLOCKS_PER_SECTOR - 1)) * BLOCK_SIZE;
        bytes[..4].copy_from_slice(&data[..4]);
        bytes[4] = i as u8;
        let key_a = &u64::to_le_bytes(key_a(&bytes))[..6];
        data[curr_block..curr_block + 6].copy_from_slice(key_a);
        data[curr_block + 6..curr_block + 10].copy_from_slice(&[0x7F, 0x0F, 0x08, 0x69]);
    }
}

fn write_ones(data: &mut [u8; NUM_BYTES]) {
    // Area counters must be updated s.t. we can modify a new skylander directly
    data[0x89] = 0x01;
    data[0x112] = 0x01;
    
    // To be considered in games after SSA
    data[0x93] = 0x01;
    data[0x96] = 0x01;
    data[0x253] = 0x01;
    data[0x256] = 0x01;
}

#[test]
pub fn test_skylander_file_io() {
    const FILE_1: &str = "./test1.sky";
    const FILE_2: &str = "./test2.sky";

    let sky1 = Skylander::new(Character::TriggerHappy, Variant::Series3, Some([0x20, 0x24, 0x49, 0x12]));
    sky1.save_to_filename(FILE_1).expect("couldn't save file");

    let sky2 = Skylander::from_filename(FILE_1).expect("couldn't read file");
    sky2.save_to_filename(FILE_2).expect("couldn't save file");

    let mut data_1 = [0u8; NUM_BYTES];
    let mut data_2 = [0u8; NUM_BYTES];
    let mut file_1 = File::open(FILE_1).expect("couldn't open file");
    let mut file_2 = File::open(FILE_2).expect("couldn't open file");

    file_1.read_exact(&mut data_1).expect("couldn't read file");
    file_2.read_exact(&mut data_2).expect("couldn't read file");

    assert_eq!(&data_1, &data_2);

    fs::remove_file(Path::new(FILE_1)).expect("couldn't delete file");
    fs::remove_file(Path::new(FILE_2)).expect("couldn't delete file");
}