use crc;
use aes::Aes128;
use block_modes::{block_padding::ZeroPadding, BlockMode};
use block_modes::Ecb;
use md_5::{Md5, Digest};

type Aes128Ecb = Ecb<Aes128, ZeroPadding>;

const BLOCK_SIZE: usize = 16;
const BLOCKS_PER_SECTOR: usize = 4;
const NUM_SECTORS: usize = 16;

/// The number of bytes that a Skylander figure (Mifare 1K NFC tag) stores
pub const NUM_BYTES: usize = BLOCK_SIZE * BLOCKS_PER_SECTOR * NUM_SECTORS;
const HASH_CONST: &[u8] = &[
    0x20, 0x43, 0x6F, 0x70, 0x79, 0x72, 0x69, 0x67, 0x68, 0x74, 0x20, 0x28, 0x43, 0x29, 0x20, 0x32,
    0x30, 0x31, 0x30, 0x20, 0x41, 0x63, 0x74, 0x69, 0x76, 0x69, 0x73, 0x69, 0x6F, 0x6E, 0x2E, 0x20,
    0x41, 0x6C, 0x6C, 0x20, 0x52, 0x69, 0x67, 0x68, 0x74, 0x73, 0x20, 0x52, 0x65, 0x73, 0x65, 0x72,
    0x76, 0x65, 0x64, 0x2E, 0x20
    ];
    const CRC16_CCITT_FALSE: crc::Algorithm<u16> = crc::Algorithm {
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

/// Generates an empty Skylander based on the character and variant provided
/// (not guaranteed that character and variant match -- improper matches will lead to "special" tag in-game).
/// Returns: figure data
/// - figure will have an nuid that is provided (default is 00 00 00 00).
/// - figure will be reset, but ownership is not taken automatically when used.
pub fn new_skylander(character: u16, variant: u16, nuid: Option<[u8; 4]>) -> [u8; NUM_BYTES] {
    let mut data = [0u8; NUM_BYTES];
    let uid = match nuid {
        Some(v) => v,
        None => [0u8; 4]
    };
    let mut bcc = uid[0];
    for i in &uid[1..] {
        bcc ^= *i;
    }

    data[0..4].copy_from_slice(&uid);
    data[4] = bcc;

    // SAK
    data[5] = 0x81;
    
    // ATQA
    data[6..=7].copy_from_slice(&[0x01, 0x0F]);
    debug_assert!(data[6] == 0x01 && data[7] == 0x0F);

    // Character
    data[BLOCK_SIZE..=BLOCK_SIZE + 1].copy_from_slice(&character.to_le_bytes());

    // Variant
    data[BLOCK_SIZE + 0xC..=BLOCK_SIZE + 0xD].copy_from_slice(&variant.to_le_bytes());

    let crc = crc::Crc::<u16>::new(&CRC16_CCITT_FALSE);
    let checksum = crc.checksum(&data[0..0x1E]);
    data[0x1E..=0x1F].copy_from_slice(&u16::to_le_bytes(checksum));

    // Sector 0 trailer
    data[(3 * BLOCK_SIZE)..(3 * BLOCK_SIZE + 10)]
        .copy_from_slice(&[0x4B, 0x0B, 0x20, 0x10, 0x7C, 0xCB, 0x0F, 0x0F, 0x0F, 0x69]);

    // Sectors 1 through 15 trailers
    for i in 1..NUM_SECTORS {
        let mut bytes = [0u8; 5];
        let curr_block = (i * BLOCKS_PER_SECTOR + (BLOCKS_PER_SECTOR - 1)) * BLOCK_SIZE;
        bytes[0..4].copy_from_slice(&data[0..4]);
        bytes[4] = i as u8;
        let key_a = &u64::to_le_bytes(key_a(&bytes))[0..6];
        data[curr_block..curr_block + 6].copy_from_slice(key_a);
        data[curr_block + 6..curr_block + 10].copy_from_slice(&[0x7F, 0x0F, 0x08, 0x69]);
    }

    // Area counters must be updated s.t. we can modify a new skylander directly
    data[0x89] = 0x01;
    data[0x112] = 0x01;

    // Because we update area counters, we have to make the data consistent
    // (although this is not technically a brand-new skylander)
    calculate_checksums(&mut data);
    encryption_skylander(&mut data, true);

    data
}

/// Encrypts or decrypts the 1K byte Skylander data. Does not check the validity of sector 0 data,
/// nor of the data blocks (checksums must be calculated before encryption and after decryption)
/// - if encrypt is true, then the data in Sectors 1 through 15 (excl. sector trailers) will be encrypted,
///   decrypted if false
/// writes in-place
pub fn encryption_skylander(data: &mut [u8; NUM_BYTES], encrypt: bool) {
    let mut seed = [0u8; 0x56];
    seed[0..0x20].copy_from_slice(&data[0..0x20]);
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