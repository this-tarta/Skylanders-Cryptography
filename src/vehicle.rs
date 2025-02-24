use crate::skyutils::*;

extend_skylander_base!(Vehicle);

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mod {
    First = 0x0,
    Second = 0x1,
    Third = 0x2,
    Fourth = 0x3
}

impl TryFrom<u8> for Mod {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x0 => Ok(Mod::First),
            0x1 => Ok(Mod::Second),
            0x2 => Ok(Mod::Third),
            0x3 => Ok(Mod::Fourth),
            _ => Err("Unknown Type")
        }
    }
}

impl std::fmt::Display for Mod {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Vehicle {
    pub fn set_performance_upgrade(&mut self, upg: Mod) {
        self.skylander.write_ones();
        let spec = match self.get_speciality_mod() {
            Ok(m) => m,
            _ => Mod::First
        };
        self.set_bytes(AREA_BOUNDS[0].0 + 0x4E, &[upg as u8 | ((spec as u8) << 4)]);
        self.set_bytes(AREA_BOUNDS[1].0 + 0x4E, &[upg as u8 | ((spec as u8) << 4)]);
    }

    pub fn get_performance_upgrade(&self) -> Result<Mod, &str> {
        let area = if self.skylander.used()[(AREA_BOUNDS[0].0 + 0x4E) / BLOCK_SIZE] { 0 } else { 1 };
        Mod::try_from(self.skylander.data()[AREA_BOUNDS[area].0 + 0x4E] & 0xF)
    }

    pub fn set_speciality_mod(&mut self, upg: Mod) {
        self.skylander.write_ones();
        let perf = match self.get_performance_upgrade() {
            Ok(m) => m,
            _ => Mod::First
        };
        self.set_bytes(AREA_BOUNDS[0].0 + 0x4E, &[((upg as u8) << 4) | perf as u8]);
        self.set_bytes(AREA_BOUNDS[1].0 + 0x4E, &[((upg as u8) << 4) | perf as u8]);
    }

    pub fn get_speciality_mod(&self) -> Result<Mod, &str> {
        let area = if self.skylander.used()[(AREA_BOUNDS[0].0 + 0x4E) / BLOCK_SIZE] { 0 } else { 1 };
        Mod::try_from((self.skylander.data()[AREA_BOUNDS[area].0 + 0x4E] >> 4) & 0xF)
    }

    pub fn set_horn(&mut self, horn: Mod) {
        self.skylander.write_ones();
        self.set_bytes(AREA_BOUNDS[0].0 + 0x4F, &[horn as u8]);
        self.set_bytes(AREA_BOUNDS[1].0 + 0x4F, &[horn as u8]);
    }

    pub fn get_horn(&self) -> Result<Mod, &str> {
        let area = if self.skylander.used()[(AREA_BOUNDS[0].0 + 0x4E) / BLOCK_SIZE] { 0 } else { 1 };
        Mod::try_from(self.skylander.data()[AREA_BOUNDS[area].0 + 0x4F])
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

    pub fn set_upgrades(&mut self, shield: u8, weapon: u8) {
        assert!(shield <= 5 && weapon <= 5);
        self.skylander.write_ones();

        let bitmap = (1u16 << shield as u16) - 1 | ((1u16 << weapon as u16) - 1) << 5;
        let bytes = bitmap.to_le_bytes();
        self.set_bytes(AREA_BOUNDS[0].0 + BLOCK_SIZE, &bytes);
        self.set_bytes(AREA_BOUNDS[1].0 + BLOCK_SIZE, &bytes);
    }

    pub fn get_upgrades(&self) -> (u8, u8) {
        let area = if self.skylander.used()[(AREA_BOUNDS[0].0 + BLOCK_SIZE) / BLOCK_SIZE] { 0 } else { 1 };
        let mut bytes = [0u8; 2];
        bytes.copy_from_slice(&self.skylander.data()[AREA_BOUNDS[area].0 + BLOCK_SIZE..AREA_BOUNDS[area].0 + BLOCK_SIZE + 2]);
        let bitmap = u16::from_le_bytes(bytes);
        let shield = (((0x1F & bitmap) + 1) as f32).log2() as u8;
        let weapon = (((0x1F & bitmap >> 5) + 1) as f32).log2() as u8;

        (shield, weapon)
    }
}

#[test]
fn test_set_get_upgrades() {
    use crate::skyfigures::Vehicle::HotStreak;
    use crate::skyvariants::Variant;

    let mut v = Vehicle::new(Toy::Vehicle(HotStreak), Variant::Vehicle, None);
    let (shield, weapon) = v.get_upgrades();

    assert_eq!(shield, 0);
    assert_eq!(weapon, 0);

    v.set_upgrades(0, 5);

    let mut bytes = [0u8; 2];
    bytes.copy_from_slice(&v.skylander.data()[AREA_BOUNDS[0].0 + BLOCK_SIZE..AREA_BOUNDS[0].0 + BLOCK_SIZE + 2]);
    assert_eq!(&bytes, &[0xE0, 0x03]);

    let (shield, weapon) = v.get_upgrades();

    assert_eq!(shield, 0);
    assert_eq!(weapon, 5);

    v.set_upgrades(5, 0);

    let mut bytes = [0u8; 2];
    bytes.copy_from_slice(&v.skylander.data()[AREA_BOUNDS[0].0 + BLOCK_SIZE..AREA_BOUNDS[0].0 + BLOCK_SIZE + 2]);
    assert_eq!(&bytes, &[0x1F, 0x00]);

    let (shield, weapon) = v.get_upgrades();

    assert_eq!(shield, 5);
    assert_eq!(weapon, 0);

    v.set_upgrades(2, 3);

    let mut bytes = [0u8; 2];
    bytes.copy_from_slice(&v.skylander.data()[AREA_BOUNDS[0].0 + BLOCK_SIZE..AREA_BOUNDS[0].0 + BLOCK_SIZE + 2]);
    assert_eq!(&bytes, &[0xE3, 0x00]);

    let (shield, weapon) = v.get_upgrades();

    assert_eq!(shield, 2);
    assert_eq!(weapon, 3);
}