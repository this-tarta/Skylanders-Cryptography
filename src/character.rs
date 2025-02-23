use std::cmp::min;

use crate::skyutils::*;
use crate::skyhats::Hat;

/// Map of level num to xp needed to achieve it
static LEVELS: [i32; 21] = [-1, 0, 1000, 2200, 3800, 6000,
        9000, 13000, 18200, 24800, 33000, 42700, 53900,
        66600, 80800, 96500, 113700, 132400, 152600, 174300, 197500];

extend_skylander_base!(Character);

impl Character {
    /// Sets the upgrade path of the Skylander
    /// Choices are from Top, Bottom, None
    pub fn set_upgrade_path(&mut self, path: UpgradePath) {
        self.skylander.write_ones();

        let byte0 = (self.skylander.data()[AREA_BOUNDS[0].0 + BLOCK_SIZE] & !0b11u8) | (path as u8);
        let byte1 = (self.skylander.data()[AREA_BOUNDS[1].0 + BLOCK_SIZE] & !0b11u8) | (path as u8);

        self.set_bytes(AREA_BOUNDS[0].0 + BLOCK_SIZE, &[byte0]);
        self.set_bytes(AREA_BOUNDS[1].0 + BLOCK_SIZE, &[byte1]);
    }

    /// Unlocks the wowpow for characters that have it
    /// True for unlock, false for lock
    pub fn set_wowpow(&mut self, set: bool) {
        self.skylander.write_ones();

        self.set_bytes(AREA_BOUNDS[2].0 + 0x6, &[set as u8]);
        self.set_bytes(AREA_BOUNDS[3].0 + 0x6, &[set as u8]);
    }

    /// Unlocks upgrades according to bitmap (least significant bit to most significant)
    pub fn set_upgrades(&mut self, bitmap: u8) {
        const fn upgrade_loc(i: usize) -> usize {AREA_BOUNDS[i].0 + BLOCK_SIZE}
        self.skylander.write_ones();

        let upgrade_path = self.get_upgrade_path();
        let mut fullmap = (bitmap as u16) << 2;
        fullmap |= upgrade_path as u16;

        let bytes = fullmap.to_le_bytes();
        self.set_bytes(upgrade_loc(0), &bytes);
        self.set_bytes(upgrade_loc(1), &bytes);
    }

    /// Gets whether the wowpow is set (true means it is set, false means not)
    pub fn get_wowpow(&self) -> bool {
        let area = if self.skylander.used()[AREA_BOUNDS[2].0 / BLOCK_SIZE] { 2 } else { 3 };
        self.skylander.data()[AREA_BOUNDS[area].0 + 0x6] == 1u8
    }

    /// Gets the upgrade path of the figure
    pub fn get_upgrade_path(&self) -> UpgradePath {
        match (self.skylander.data()[AREA_BOUNDS[0].0 + BLOCK_SIZE] | self.skylander.data()[AREA_BOUNDS[1].0 + BLOCK_SIZE]) & 0b11 {
            0b01 => UpgradePath::Top,
            0b11 => UpgradePath::Bottom,
            _ => UpgradePath::None
        }
    }

    /// Gets the upgrades of the figure as a bitmap
    pub fn get_upgrades(&self) -> u8 {
        const fn upgrade_loc(i: usize) -> usize {AREA_BOUNDS[i].0 + BLOCK_SIZE}
        let area = if self.skylander.used()[AREA_BOUNDS[0].0 / BLOCK_SIZE] { 0 } else { 1 };
        let mut bytes = [0u8; 2];
        bytes.copy_from_slice(&self.skylander.data()[upgrade_loc(area) .. upgrade_loc(area) + 2]);
        let bitmap = u16::from_le_bytes(bytes);
        (bitmap >> 2) as u8
    }

    /// Sets gold of the Skylander to a specified value
    /// Note that in-game, the gold is capped at 65000
    pub fn set_gold(&mut self, gold: u16) {
        self.skylander.write_ones();
        self.set_bytes(AREA_BOUNDS[0].0 + 0x3, &gold.to_le_bytes());
        self.set_bytes(AREA_BOUNDS[1].0 + 0x3, &gold.to_le_bytes());
    }

    /// Sets gold of Skylander to max
    pub fn max_gold(&mut self) {
        self.set_gold(u16::MAX);
    }

    /// Returns the gold of the Skylander
    pub fn get_gold(&self) -> u16{
        let mut bytes = [0u8; 2];
        let area = if self.skylander.used()[AREA_BOUNDS[0].0 / BLOCK_SIZE] { 0 } else { 1 };
        bytes.copy_from_slice(&self.skylander.data()[AREA_BOUNDS[area].0 + 0x3..= AREA_BOUNDS[area].0 + 0x4]);
        u16::from_le_bytes(bytes)
    }
    
    /// Sets experience points of the Skylander to specified value
    /// Max experience in Spyro's Adventure is 33000 (level 10)
    ///                in Giants is 96500 (level 15)
    ///                in Swap Force and beyond is 197500 (level 20)
    /// This function will take min(xp, 197500)
    pub fn set_xp(&mut self, xp: u32) {
        self.skylander.write_ones();
        
        let xp1 = min(xp, 33000);
        let xp2 = min(xp - xp1, 63500);
        let xp3 = min(xp - xp1 - xp2, 101000);

        debug_assert!(xp1 + xp2 + xp3 == xp || xp1 + xp2 + xp3 == 197500);

        let xp1_bytes = (xp1 as u16).to_le_bytes();
        let xp2_bytes = (xp2 as u16).to_le_bytes();
        let xp3_bytes = xp3.to_le_bytes();

        self.set_bytes(AREA_BOUNDS[0].0, &xp1_bytes);
        self.set_bytes(AREA_BOUNDS[1].0, &xp1_bytes);
        
        self.set_bytes(AREA_BOUNDS[2].0 + 0x3, &xp2_bytes);
        self.set_bytes(AREA_BOUNDS[3].0 + 0x3, &xp2_bytes);
        
        self.set_bytes(AREA_BOUNDS[2].0 + 0x8, &xp3_bytes[..3]);
        self.set_bytes(AREA_BOUNDS[3].0 + 0x8, &xp3_bytes[..3]);
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

        let area1 = if self.skylander.used()[AREA_BOUNDS[0].0 / BLOCK_SIZE] { 0 } else { 1 };
        let area2 = if self.skylander.used()[AREA_BOUNDS[2].0 / BLOCK_SIZE] { 2 } else { 3 };
        
        xp1_bytes.copy_from_slice(&self.skylander.data()[AREA_BOUNDS[area1].0 ..= AREA_BOUNDS[area1].0 + 0x1]);
        xp2_bytes.copy_from_slice(&self.skylander.data()[AREA_BOUNDS[area2].0 + 0x3 ..= AREA_BOUNDS[area2].0 + 0x4]);
        xp3_bytes[..3].copy_from_slice(&self.skylander.data()[AREA_BOUNDS[area2].0 + 0x8 .. AREA_BOUNDS[area2].0 + 0xB]);

        u16::from_le_bytes(xp1_bytes) as u32 + u16::from_le_bytes(xp2_bytes) as u32 + u32::from_le_bytes(xp3_bytes)
    }

    /// Get the current level of the skylander
    pub fn get_level(&self) -> u8 {
        let xp = self.get_xp();
        let mut level = 0;
        let mut start = 0;
        let mut end = LEVELS.len() - 1;

        while start <= end {
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

    /// Sets the hat on the Skylander
    pub fn set_hat(&mut self, hat: Hat) {
        self.skylander.write_ones();

        self.set_bytes(0x94, &(hat as u16).to_le_bytes());
        self.set_bytes(0x254, &(hat as u16).to_le_bytes());
    }

    /// Gets the hat of a Skylander, returns error if not a valid hat
    pub fn get_hat(&self) -> Result<Hat, <Hat as TryFrom<u16>>::Error> {
        let mut bytes = [0u8; 2];
        let area = if self.skylander.used()[AREA_BOUNDS[0].0 / BLOCK_SIZE] { 0 } else { 1 };
        bytes.copy_from_slice(&self.skylander.data()[AREA_BOUNDS[area].0 + 0x14..= AREA_BOUNDS[area].0 + 0x15]);
        Hat::try_from(u16::from_le_bytes(bytes))
    }

}