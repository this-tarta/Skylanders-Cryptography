#![allow(unused)]

use crc;
use aes::Aes128;
use block_modes::{block_padding::ZeroPadding, BlockMode};
use block_modes::Ecb;
use md5::{Md5, Digest};
use std::cmp::min;
use std::{u16, u32};
use std::{fs::{self, File}, io::{self, Read, Seek, Write}, path::Path};
use mifare_utils::*;

use crate::skyfigures::{Character, Expansion, ImaginatorCrystal, Item, Trap, Vehicle};
use crate::skyvariants::Variant;
use crate::skyhats::Hat;

type Aes128Ecb = Ecb<Aes128, ZeroPadding>;

const BLOCK_SIZE: usize = 16;
const BLOCKS_PER_SECTOR: usize = 4;
const SECTOR_SIZE: usize = BLOCK_SIZE * BLOCKS_PER_SECTOR;
const NUM_SECTORS: usize = 16;
/// The number of bytes that a Skylander figure (Mifare 1K NFC tag) stores
const NUM_BYTES: usize = SECTOR_SIZE * NUM_SECTORS;

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
static KEY_A_SECTOR_0: &[u8; 6] = &[0x4B, 0x0B, 0x20, 0x10, 0x7C, 0xCB];

/// Map of level num to xp needed to achieve it
static LEVELS: [i32; 21] = [-1, 0, 1000, 2200, 3800, 6000,
        9000, 13000, 18200, 24800, 33000, 42700, 53900,
        66600, 80800, 96500, 113700, 132400, 152600, 174300, 197500];

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
    data: Box<[u8; NUM_BYTES]>,
    used: [bool; NUM_SECTORS],
    modified: bool
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
        data[(3 * BLOCK_SIZE)..(3 * BLOCK_SIZE + 6)]
            .copy_from_slice(KEY_A_SECTOR_0);
        data[(3 * BLOCK_SIZE + 6)..(3 * BLOCK_SIZE + 10)]
            .copy_from_slice(&[0x0F, 0x0F, 0x0F, 0x69]);
    

        calculate_key_a(&mut data);
        let mut used = [false; NUM_SECTORS];
        used[0] = true;
        
        Self { data, used, modified: false }
    }

    /// Saves the Skylander to a file
    /// Overwrites any data up to 1KB from seek start
    fn save_to_file(&self, file: &mut File) -> io::Result<()> {
        let mut data = *(self.data).clone();
        if self.modified {
            calculate_checksums(&mut data);
        }
        encryption_skylander(&mut data, &self.used, true);
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
        let used = update_used(& *data);
        encryption_skylander(&mut *data, &used, false);

        Ok(Self { data , used, modified: false })
    }

    /// Reads a Skylander from a file
    /// Does not verify the data integrity of the file
    pub fn from_filename(filename: &str) -> io::Result<Self> {
        Self::from_filepath(Path::new(filename))
    }

    /// Reads a Skylander from nfc card (or figure itself)
    /// Does not verify validity of the card -- Must be well-formed Skylander data
    pub fn from_nfc() -> Result<Self, MifareError> {
        let mut data = Box::new([0u8; NUM_BYTES]);
        let connection = MifareReader::new()?;
        let card = connection.connect(&connection.list_readers()?[0])?;

        // Sector 0
        card.authenticate_with_key(0, KEY_A_SECTOR_0, KeyType::KeyA)?;
        for i in 0..BLOCKS_PER_SECTOR {
            data[i * BLOCK_SIZE..(i + 1) * BLOCK_SIZE].copy_from_slice(&card.read_block(i as u8)?);
        }
        calculate_key_a(&mut *data);

        for i in 1..NUM_SECTORS {
            let mut key_a = [0u8; 6];
            let sector_start = i * SECTOR_SIZE;
            let sector_trailer = &data[sector_start + 3 * BLOCK_SIZE..sector_start + 3 * BLOCK_SIZE + 6];
            key_a.copy_from_slice(sector_trailer);

            let sector_hdr = i * BLOCKS_PER_SECTOR;
            card.authenticate_with_key(sector_hdr as u8, &key_a, KeyType::KeyA)?;
            for i in 0..BLOCKS_PER_SECTOR - 1 {
                let block = sector_hdr + i;
                data[block * BLOCK_SIZE..(block + 1) * BLOCK_SIZE]
                    .copy_from_slice(&card.read_block(block as u8)?);
            }
        }

        let used = update_used(& *data);
        encryption_skylander(&mut *data, &used, false);

        Ok(Self { data , used, modified: false })
    }

    /// Saves a Skylander from nfc card (or figure itself)
    /// Does not verify validity of the card -- Must have well-formed Sector 0 and sector trailers
    pub fn save_to_nfc(&self) -> Result<(), MifareError> {
        let mut data = *(self.data).clone();
        if self.modified {
            calculate_checksums(&mut data);
        }
        encryption_skylander(&mut data, &self.used, true);

        let connection = MifareReader::new()?;
        let card = connection.connect(&connection.list_readers()?[0])?;

        for i in 1..NUM_SECTORS {
            let mut key_a = [0u8; 6];
            let sector_start = i * SECTOR_SIZE;
            let sector_trailer = &data[sector_start + 3 * BLOCK_SIZE..sector_start + 3 * BLOCK_SIZE + 6];
            key_a.copy_from_slice(sector_trailer);

            for j in 0..BLOCKS_PER_SECTOR - 1 {
                let block = (i * BLOCKS_PER_SECTOR + j);
                let mut copy = [0u8; BLOCK_SIZE];
                copy.copy_from_slice(&data[block * BLOCK_SIZE..(block + 1) * BLOCK_SIZE]);
                card.authenticate_with_key(block as u8, &key_a, KeyType::KeyA)?;
                card.write_block(block as u8, &copy)?;
            }
        }

        Ok(())
    }

    /// Sets gold of the Skylander to a specified value
    /// Note that in-game, the gold is capped at 65000
    pub fn set_gold(&mut self, gold: u16) {
        self.used[AREA_BOUNDS[0].0 / SECTOR_SIZE] = true;
        self.used[AREA_BOUNDS[1].0  / SECTOR_SIZE] = true;
        write_ones(&mut *self.data);
        self.modified = true;

        self.data[AREA_BOUNDS[0].0 + 0x3..= AREA_BOUNDS[0].0 + 0x4].copy_from_slice(&gold.to_le_bytes());
        self.data[AREA_BOUNDS[1].0 + 0x3..= AREA_BOUNDS[1].0 + 0x4].copy_from_slice(&gold.to_le_bytes());
    }

    /// Sets gold of Skylander to max
    pub fn max_gold(&mut self) {
        self.set_gold(u16::MAX);
    }

    /// Returns the gold of the Skylander
    pub fn get_gold(&self) -> u16{
        let mut bytes = [0u8; 2];
        let area = if self.used[AREA_BOUNDS[0].0 / SECTOR_SIZE] { 0 } else { 1 };
        bytes.copy_from_slice(&self.data[AREA_BOUNDS[area].0 + 0x3..= AREA_BOUNDS[area].0 + 0x4]);
        u16::from_le_bytes(bytes)
    }
    
    /// Sets experience points of the Skylander to specified value
    /// Max experience in Spyro's Adventure is 33000 (level 10)
    ///                in Giants is 96500 (level 15)
    ///                in Swap Force and beyond is 197500 (level 20)
    /// This function will take min(xp, 197500)
    pub fn set_xp(&mut self, xp: u32) {
        self.used[AREA_BOUNDS[0].0 / SECTOR_SIZE] = true;
        self.used[AREA_BOUNDS[1].0  / SECTOR_SIZE] = true;
        self.used[AREA_BOUNDS[2].0  / SECTOR_SIZE] = true;
        self.used[AREA_BOUNDS[3].0  / SECTOR_SIZE] = true;
        write_ones(&mut *self.data);
        self.modified = true;
        
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

    /// Sets the level of the Skylander
    /// level must be in [1, 20]
    pub fn set_level(&mut self, level: u8) {
        assert!(level >= 1 && level <= 20);
        self.set_xp(LEVELS[level as usize] as u32);
    }

    /// Get the current experience of the Skylander
    pub fn get_xp(&self) -> u32 {
        let mut xp1_bytes = [0u8; 2];
        let mut xp2_bytes = [0u8; 2];
        let mut xp3_bytes = [0u8; 4];

        let area1 = if self.used[AREA_BOUNDS[0].0 / SECTOR_SIZE] { 0 } else { 1 };
        let area2 = if self.used[AREA_BOUNDS[2].0 / SECTOR_SIZE] { 2 } else { 3 };
        
        xp1_bytes.copy_from_slice(&self.data[AREA_BOUNDS[area1].0 ..= AREA_BOUNDS[area1].0 + 0x1]);
        xp2_bytes.copy_from_slice(&self.data[AREA_BOUNDS[area2].0 + 0x3 ..= AREA_BOUNDS[area2].0 + 0x4]);
        xp3_bytes[..3].copy_from_slice(&self.data[AREA_BOUNDS[area2].0 + 0x8 .. AREA_BOUNDS[area2].0 + 0xB]);

        u16::from_le_bytes(xp1_bytes) as u32 + u16::from_le_bytes(xp2_bytes) as u32 + u32::from_le_bytes(xp3_bytes)
    }

    /// Get the current level of the skylander
    pub fn get_level(&self) -> u8 {
        let xp = self.get_xp();
        let mut level = 0;
        let mut start = 0;
        let mut end = LEVELS.len() - 1;

        while (start <= end) {
            let mid = end - (end - start) / 2;
            if LEVELS[mid] < xp as i32 {
                level = mid;
                start = mid + 1;
            } else if LEVELS[mid] == xp as i32 {
                level = mid;
                break;
            } else {
                end = mid - 1;
            }
        }

        level as u8
    }

    /// Clears all data from the skylander
    pub fn clear(&mut self) {
        for i in 1..NUM_SECTORS {
            let sector_start = i * SECTOR_SIZE;
            let sector_trailer = sector_start + (BLOCKS_PER_SECTOR - 1) * BLOCK_SIZE;
            self.data[sector_start..sector_trailer].copy_from_slice(&[0u8; (BLOCKS_PER_SECTOR - 1) * BLOCK_SIZE]);
        }

        self.used = [false; NUM_SECTORS];
        self.used[0] = true;
        self.modified = false;
    }

    /// Sets the hat on the Skylander
    pub fn set_hat(&mut self, hat: Hat) {
        self.used[0x94 / SECTOR_SIZE] = true;
        self.used[0x254 / SECTOR_SIZE] = true;
        write_ones(&mut *self.data);
        self.modified = true;
        
        self.data[0x94..=0x95].copy_from_slice(&(hat as u16).to_le_bytes());
        self.data[0x254..=0x255].copy_from_slice(&(hat as u16).to_le_bytes());
    }

    /// Gets the hat of a Skylander, returns error if not a valid hat
    pub fn get_hat(&self) -> Result<Hat, <Hat as TryFrom<u16>>::Error> {
        let mut bytes = [0u8; 2];
        let area = if self.used[AREA_BOUNDS[0].0 / SECTOR_SIZE] { 0 } else { 1 };
        bytes.copy_from_slice(&self.data[AREA_BOUNDS[area].0 + 0x14..= AREA_BOUNDS[area].0 + 0x15]);
        Hat::try_from(u16::from_le_bytes(bytes))
    }

    /// Gets what the figure is; returns Unknown(u16) if it is not a
    /// Character, Trap, Vehicle, Item, Expansion, or Imaginator Crystal
    /// where u16 is the id of the figure
    pub fn get_figure(&self) -> Toy {
        let mut bytes = [0u8; 2];
        bytes.copy_from_slice(&self.data[BLOCK_SIZE..=BLOCK_SIZE + 1]);
        let id = u16::from_le_bytes(bytes);
        match Character::try_from(id) {
            Ok(c) => return Toy::Character(c),
            _ => ()
        };
        match Trap::try_from(id) {
            Ok(t) => return Toy::Trap(t),
            _ => ()
        };
        match Vehicle::try_from(id) {
            Ok(v) => return Toy::Vehicle(v),
            _ => ()
        };
        match Item::try_from(id) {
            Ok(i) => return Toy::Item(i),
            _ => ()
        };
        match Expansion::try_from(id) {
            Ok(e) => return Toy::Expansion(e),
            _ => ()
        }
        match ImaginatorCrystal::try_from(id) {
            Ok(i) => return Toy::ImaginatorCrystal(i),
            _ => ()
        }
        Toy::Unknown(id)
    }

    /// Gets the Variant of a Skylander, returns error if not a valid Variant
    pub fn get_variant(&self) -> Result<Variant, <Variant as TryFrom<u16>>::Error> {
        let mut bytes = [0u8; 2];
        bytes.copy_from_slice(&self.data[BLOCK_SIZE + 0xC..=BLOCK_SIZE + 0xD]);
        Variant::try_from(u16::from_le_bytes(bytes))
    }

    /// Sets the upgrade path of the Skylander
    /// Choices are from Top, Bottom, None
    pub fn set_upgrade_path(&mut self, path: UpgradePath) {
        self.used[AREA_BOUNDS[0].0 / SECTOR_SIZE] = true;
        self.used[AREA_BOUNDS[1].0 / SECTOR_SIZE] = true;
        write_ones(&mut *self.data);
        self.modified = true;
        
        self.data[AREA_BOUNDS[0].0 + BLOCK_SIZE] &= (0xFFu8 ^ 0b11u8);
        self.data[AREA_BOUNDS[0].0 + BLOCK_SIZE] |= path as u8;
        self.data[AREA_BOUNDS[1].0 + BLOCK_SIZE] &= (0xFFu8 ^ 0b11u8);
        self.data[AREA_BOUNDS[1].0 + BLOCK_SIZE] |= path as u8;
    }

    /// Unlocks the wowpow for characters that have it
    /// True for unlock, false for lock
    pub fn set_wowpow(&mut self, set: bool) {
        self.used[AREA_BOUNDS[2].0 / SECTOR_SIZE] = true;
        self.used[AREA_BOUNDS[3].0 / SECTOR_SIZE] = true;
        write_ones(&mut *self.data);
        self.modified = true;
        
        self.data[AREA_BOUNDS[2].0 + 0x6] = set as u8;
        self.data[AREA_BOUNDS[3].0 + 0x6] = set as u8;
    }

    /// Unlocks upgrades according to bitmap (least significant bit to most significant)
    pub fn set_upgrades(&mut self, bitmap: u8) {
        const fn upgrade_loc(i: usize) -> usize {AREA_BOUNDS[i].0 + BLOCK_SIZE};
        self.used[upgrade_loc(0) / SECTOR_SIZE] = true;
        self.used[upgrade_loc(1) / SECTOR_SIZE] = true;
        write_ones(&mut *self.data);
        self.modified = true;

        let upgrade_path = self.get_upgrade_path();
        let mut fullmap: u16 = 0;
        fullmap = (bitmap as u16) << 2;
        fullmap |= upgrade_path as u16;

        let bytes = fullmap.to_le_bytes();
        self.data[upgrade_loc(0) .. upgrade_loc(0) + 2].copy_from_slice(&bytes);
        self.data[upgrade_loc(1) .. upgrade_loc(1) + 2].copy_from_slice(&bytes);
    }

    /// Gets whether the wowpow is set (true means it is set, false means not)
    pub fn get_wowpow(&self) -> bool {
        let area = if self.used[AREA_BOUNDS[2].0 / SECTOR_SIZE] { 2 } else { 3 };
        self.data[AREA_BOUNDS[area].0 + 0x6] == 1u8
    }

    /// Gets the upgrade path of the figure
    pub fn get_upgrade_path(&self) -> UpgradePath {
        match (self.data[AREA_BOUNDS[0].0 + BLOCK_SIZE] | self.data[AREA_BOUNDS[1].0 + BLOCK_SIZE]) & 0b11 {
            0b01 => UpgradePath::Top,
            0b11 => UpgradePath::Bottom,
            _ => UpgradePath::None
        }
    }

    /// Gets the upgrades of the figure as a bitmap
    pub fn get_upgrades(&self) -> u8 {
        const fn upgrade_loc(i: usize) -> usize {AREA_BOUNDS[i].0 + BLOCK_SIZE};
        let area = if self.used[AREA_BOUNDS[0].0 / SECTOR_SIZE] { 0 } else { 1 };
        let mut bytes = [0u8; 2];
        bytes.copy_from_slice(&self.data[upgrade_loc(area) .. upgrade_loc(area) + 2]);
        let bitmap = u16::from_le_bytes(bytes);
        (bitmap >> 2) as u8
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Toy {
    Character(Character),
    Trap(Trap),
    Vehicle(Vehicle),
    Item(Item),
    Expansion(Expansion),
    ImaginatorCrystal(ImaginatorCrystal),
    Unknown(u16)
}

impl std::fmt::Display for Toy {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
pub enum UpgradePath {
    Top = 0b01,
    Bottom = 0b11,
    None = 0b00
}

fn update_used(data: &[u8; NUM_BYTES]) -> [bool; NUM_SECTORS] {
    let mut used = [false; NUM_SECTORS];
    used[0] = true;
    for i in 0..NUM_SECTORS {
        let sector_start = i * SECTOR_SIZE;
        for byte in &data[sector_start.. sector_start + SECTOR_SIZE - BLOCK_SIZE] {
            if *byte != 0x00u8 {
                used[i] = true;
                break;
            }
        }
    }
    used
}

/// Encrypts or decrypts the 1K byte Skylander data. Does not check the validity of sector 0 data,
/// nor of the data blocks (checksums must be calculated before encryption and after decryption)
/// - if encrypt is true, then the data in Sectors 1 through 15 (excl. sector trailers) will be encrypted,
///   decrypted if false
/// writes in-place
fn encryption_skylander(data: &mut [u8; NUM_BYTES], used: &[bool; NUM_SECTORS], encrypt: bool) {
    let mut seed = [0u8; 0x56];
    seed[..0x20].copy_from_slice(&data[..0x20]);
    seed[0x21..].copy_from_slice(HASH_CONST);

    for i in 1..NUM_SECTORS {
        if !used[i] {
            continue;
        }
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
fn calculate_checksums(data: &mut [u8; NUM_BYTES]) {
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
    data[0x249] = 0x00;
    data[0x112] = 0x01;
    data[0x2D2] = 0x00;
    
    // To be considered in games after SSA
    data[0x93] = 0x01;
    data[0x96] = 0x01;
    data[0x253] = 0x01;
    data[0x256] = 0x01;
}

#[test]
fn test_skylander_file_io() {
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

    fs::remove_file(Path::new(FILE_1)).expect("couldn't delete file");
    fs::remove_file(Path::new(FILE_2)).expect("couldn't delete file");

    assert_eq!(&data_1, &data_2);
}

#[test]
fn dump_decrypted_skylander_from_file() {
    const FILE_1: &str = "../Skylanders_Files/Tests/test2.sky"; // change this
    const FILE_2: &str = "../Skylanders_Files/Tests/test2_dec.sky";

    let sky1 = Skylander::from_filename(FILE_1).expect("couldn't read file");
    let mut file_2 = File::create(FILE_2).expect("couldn't create file");
    file_2.write_all(& *sky1.data).expect("Couldn't write file 2");
}

#[test]
fn encrypt_decrypted_skylander_dump() {
    const FILE_1: &str = "../Skylanders_Files/Tests/test1_dec.sky"; // change this
    const FILE_2: &str = "../Skylanders_Files/Tests/test2.sky";

    let mut file_1 = File::open(FILE_1).expect("Couldn't open file");
    let mut data = [0u8; NUM_BYTES];
    file_1.read_exact(&mut data).expect("Could not read file");

    let sky1 = Skylander {data: Box::new(data), modified: true, used: [true; NUM_SECTORS] };
    sky1.save_to_filename(FILE_2).expect("couldn't write to file");
}