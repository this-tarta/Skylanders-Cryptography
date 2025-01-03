use std::{sync::LazyLock, collections::HashMap};
pub use strum::IntoEnumIterator;
use strum_macros::EnumIter;

static VARIANT_INDEX: LazyLock<HashMap<u16, Variant>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    for var in Variant::iter() {
        m.insert(var as u16, var);
    }
    m
});

#[repr(u16)]
#[derive(Clone, Copy)]
enum Game {
    Spyros = 0x0000,
    Giants = 0x1000,
    SwapForce = 0x2000,
    TrapTeam = 0x3000,
    Superchargers = 0x4000,
    Imaginators = 0x5000
}

#[repr(u16)]
enum VariantTypes {
    WowPow = 0x0800,
    AltDeco = 0x0400,
    Lightcore = 0x0200,
    Supercharger = 0x0100,
    Default = 0x0000
}

#[repr(u16)]
#[derive(Clone, Copy)]
#[allow(unused)]
enum DecoID {
	Normal = 0,
	Repose1 = 1,
	AlternateDeco = 2,
	Legendary = 3,
	Event = 4,
	Repose2 = 5,
	LightDirect = 6,
	LightStored = 7,
	LightEnhanced = 8,
	Repose3 = 9,
	Repose4 = 10,
	Repose5 = 11,
	Valentines = 12,
	Easter = 13,
	Winter = 14,
	Virtual = 15,
	Premium = 16,
	GlowDark = 17,
	StoneStatue = 18,
	GlitterSpectrum = 19,
	TreasureHunt2012 = 20,
	Halloween = 21,
	TreasureHunt2013 = 22,
	ColorShift = 23,
	WiiU = 24,
	BestBuy = 25,
	FritoLay = 26,
	TreasureHunt2014 = 29,
	TreasureHunt2015 = 30,
	Mobile = 31,
}

const fn get_variant(game: Game, decoid: DecoID, bf: u16) -> u16 {
    (game as u16) | (decoid as u16) | (bf)
}

#[repr(u16)]
#[derive(Clone, Copy, Debug, EnumIter)]
pub enum Variant {
    // Characters from Spyro's
    Series1 = get_variant(Game::Spyros, DecoID::Normal, VariantTypes::Default as u16),
    Series2 = get_variant(Game::Giants, DecoID::Repose1, VariantTypes::WowPow as u16),
    Series3 = get_variant(Game::SwapForce, DecoID::Repose2, VariantTypes::WowPow as u16),
    Series4 = get_variant(Game::TrapTeam, DecoID::Repose3, VariantTypes::WowPow as u16),

    // Characters from Giants
    GiantCore1 = get_variant(Game::Giants, DecoID::Normal, VariantTypes::Default as u16),
    GiantCore2 = get_variant(Game::SwapForce, DecoID::Repose1, VariantTypes::WowPow as u16),
    GiantCore3 = get_variant(Game::TrapTeam, DecoID::Repose2, VariantTypes::WowPow as u16),

    // Characters from Swap Force
    SwapForce1 = get_variant(Game::SwapForce, DecoID::Normal, VariantTypes::Default as u16),
    SwapForce2 = get_variant(Game::TrapTeam, DecoID::Repose1, VariantTypes::WowPow as u16),

    // Characters from Trap Team
    TrapTeam = get_variant(Game::TrapTeam, DecoID::Normal, VariantTypes::Default as u16),

    // Characters from Superchargers
    Superchargers = get_variant(Game::Superchargers, DecoID::Normal, VariantTypes::Supercharger as u16),
    Vehicle = get_variant(Game::Superchargers, DecoID::Normal, VariantTypes::Default as u16),

    // Characters from Imaginators
    Imaginators = get_variant(Game::Imaginators, DecoID::Normal, VariantTypes::Default as u16),

    // Lightcores
    GiantsLightcore = get_variant(Game::Giants, DecoID::LightDirect, VariantTypes::Lightcore as u16),
    SwapForceLightcore = get_variant(Game::SwapForce, DecoID::LightDirect, VariantTypes::Lightcore as u16),

    // Eon's Elite
    TTElite = get_variant(Game::TrapTeam, DecoID::Premium, VariantTypes::WowPow as u16),
    SCElite = get_variant(Game::Superchargers, DecoID::Premium, VariantTypes::WowPow as u16),

    // AltDecos
    GiantAlt = get_variant(Game::Giants, DecoID::AlternateDeco, VariantTypes::Lightcore as u16 | VariantTypes::AltDeco as u16),
    GiantReturningCoreAlt = get_variant(Game::Giants, DecoID::AlternateDeco, VariantTypes::WowPow as u16 | VariantTypes::AltDeco as u16),
    GiantNewCoreAlt = get_variant(Game::Giants, DecoID::AlternateDeco, VariantTypes::AltDeco as u16),
    SFNewAlt = get_variant(Game::SwapForce, DecoID::AlternateDeco, VariantTypes::AltDeco as u16),
    SFReturningAlt = get_variant(Game::SwapForce, DecoID::AlternateDeco, VariantTypes::AltDeco as u16 | VariantTypes::WowPow as u16),
    TTReturningAlt = get_variant(Game::TrapTeam, DecoID::AlternateDeco, VariantTypes::AltDeco as u16 | VariantTypes::WowPow as u16),
    TTNewAlt = get_variant(Game::TrapTeam, DecoID::AlternateDeco, VariantTypes::AltDeco as u16),
    SCAlt = get_variant(Game::Superchargers, DecoID::AlternateDeco, VariantTypes::AltDeco as u16 | VariantTypes::Supercharger as u16),
    SCHalloween = get_variant(Game::Superchargers, DecoID::Halloween, VariantTypes::AltDeco as u16 | VariantTypes::Supercharger as u16),
    SCWinter = get_variant(Game::Superchargers, DecoID::Winter, VariantTypes::AltDeco as u16 | VariantTypes::Supercharger as u16),
    SCEaster = get_variant(Game::Superchargers, DecoID::Easter, VariantTypes::AltDeco as u16 | VariantTypes::Supercharger as u16),
    VehicleAlt = get_variant(Game::Superchargers, DecoID::AlternateDeco, VariantTypes::AltDeco as u16),
    VehicleHalloween = get_variant(Game::Superchargers, DecoID::Halloween, VariantTypes::AltDeco as u16),
    VehicleWinter = get_variant(Game::Superchargers, DecoID::Winter, VariantTypes::AltDeco as u16),
    VehicleEaster = get_variant(Game::Superchargers, DecoID::Easter, VariantTypes::AltDeco as u16),
    ImaginatorsAlt = get_variant(Game::Imaginators, DecoID::AlternateDeco, VariantTypes::AltDeco as u16),
    ImaginatorsHalloween = get_variant(Game::Imaginators, DecoID::Halloween, VariantTypes::AltDeco as u16),
    ImaginatorsWinter = get_variant(Game::Imaginators, DecoID::Winter, VariantTypes::AltDeco as u16),
    ImaginatorsEaster = get_variant(Game::Imaginators, DecoID::Easter, VariantTypes::AltDeco as u16),

    // Legendaries (SG and beyond)
    GiantLegendary = get_variant(Game::Giants, DecoID::Legendary, VariantTypes::Lightcore as u16 | VariantTypes::AltDeco as u16),
    GiantReturningCoreLegendary = get_variant(Game::Giants, DecoID::Legendary, VariantTypes::WowPow as u16 | VariantTypes::AltDeco as u16),
    GiantNewCoreLegendary = get_variant(Game::Giants, DecoID::Legendary, VariantTypes::AltDeco as u16),
    SFNewLegendary = get_variant(Game::SwapForce, DecoID::Legendary, VariantTypes::AltDeco as u16),
    SFReturningLegendary = get_variant(Game::SwapForce, DecoID::Legendary, VariantTypes::AltDeco as u16 | VariantTypes::WowPow as u16),
    TTReturningLegendary = get_variant(Game::TrapTeam, DecoID::Legendary, VariantTypes::AltDeco as u16 | VariantTypes::WowPow as u16),
    TTNewLegendary = get_variant(Game::TrapTeam, DecoID::Legendary, VariantTypes::AltDeco as u16),
    SCLegendary = get_variant(Game::Superchargers, DecoID::Legendary, VariantTypes::AltDeco as u16 | VariantTypes::Supercharger as u16),
    VehicleLegendary = get_variant(Game::Superchargers, DecoID::Legendary, VariantTypes::AltDeco as u16),
    ImaginatorsLegendary = get_variant(Game::Imaginators, DecoID::Legendary, VariantTypes::AltDeco as u16)
}

impl TryFrom<u16> for Variant {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match VARIANT_INDEX.get(&value) {
            Some(&exp) => Ok(exp),
            None => Err("Invalid Variant value")
        }
    }
}

impl std::fmt::Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}