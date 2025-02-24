use crate::skyutils::*;

extend_skylander_base!(Vehicle);

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PerformanceUpgrade {
    First = 0x30,
    Second = 0x31,
    Third = 0x32,
    Fourth = 0x33
}

impl TryFrom<u8> for PerformanceUpgrade {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x30 => Ok(PerformanceUpgrade::First),
            0x31 => Ok(PerformanceUpgrade::Second),
            0x32 => Ok(PerformanceUpgrade::Third),
            0x33 => Ok(PerformanceUpgrade::Fourth),
            _ => Err("Unknown Type")
        }
    }
}

impl std::fmt::Display for PerformanceUpgrade {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Vehicle {
    pub fn set_performance_upgrade(&mut self, upg: PerformanceUpgrade) {
        self.skylander.write_ones();
        self.set_bytes(AREA_BOUNDS[0].0 + 0x4E, &[upg as u8]);
        self.set_bytes(AREA_BOUNDS[1].0 + 0x4E, &[upg as u8]);
    }

    pub fn get_performance_upgrade(&self) -> Result<PerformanceUpgrade, &str> {
        let area = if self.skylander.used()[(AREA_BOUNDS[0].0 + 0x4E) / BLOCK_SIZE] { 0 } else { 1 };
        PerformanceUpgrade::try_from(self.skylander.data()[AREA_BOUNDS[area].0 + 0x4E])
    }

    pub fn set_gears(&mut self, gears: u16) {
        let bytes = gears.to_le_bytes();
        self.skylander.write_ones();
        self.set_bytes(AREA_BOUNDS[2].0 + 0x8, &bytes);
        self.set_bytes(AREA_BOUNDS[3].0 + 0x8, &bytes);
    }

    pub fn get_gears(&self) -> u16 {
        let area = if self.skylander.used()[(AREA_BOUNDS[2].0 + 0x8) / BLOCK_SIZE] { 2 } else { 3 };
        let mut bytes = [0u8; 2];
        bytes.copy_from_slice(&self.skylander.data()[AREA_BOUNDS[area].0 + 0x8..=AREA_BOUNDS[area].0 + 0x9]);
        u16::from_le_bytes(bytes)
    }
}