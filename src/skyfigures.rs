
use std::{collections::HashMap, hash::Hash, sync::LazyLock};
pub use strum::IntoEnumIterator;
use strum_macros::EnumIter;

static CHARACTER_INDEX: LazyLock<HashMap<u16, Character>> = make_index();
static ITEM_INDEX: LazyLock<HashMap<u16, Item>> = make_index();
static CRYSTAL_INDEX: LazyLock<HashMap<u16, ImaginatorCrystal>> = make_index();
static TRAP_INDEX: LazyLock<HashMap<u16, Trap>> = make_index();
static EXPANSION_INDEX: LazyLock<HashMap<u16, Expansion>> = make_index();
static VEHICLE_INDEX: LazyLock<HashMap<u16, Vehicle>> = make_index();

const fn make_index<S, T>() -> LazyLock<HashMap<S, T>>
        where T: IntoEnumIterator + Into<S> + Copy, S: Eq + Hash {
    LazyLock::new(|| {
        let mut m = HashMap::new();
        for t in T::iter() {
            m.insert(t.into(), t);
        }
        m
    })
}

#[repr(u16)]
#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq)]
pub enum Character {
    // Spyro's
    Whirlwind = 0,
    SonicBoom = 1,
    Warnado = 2,
    LightningRod = 3,
    Bash = 4,
    Terrafin = 5,
    DinoRang = 6,
    PrismBreak = 7,
    Sunburn = 8,
    Eruptor = 9,
    Ignitor = 10,
    Flameslinger = 11,
    Zap = 12,
    WhamShell = 13,
    GillGrunt = 14,
    SlamBam = 15,
    Spyro = 16,
    Voodood = 17,
    DoubleTrouble = 18,
    TriggerHappy = 19,
    Drobot = 20,
    DrillSergeant = 21,
    Boomer = 22,
    WreckingBall = 23,
    Camo = 24,
    Zook = 25,
    StealthElf = 26,
    StumpSmash = 27,
    DarkSpyro = 28,
    Hex = 29,
    ChopChop = 30,
    GhostRoaster = 31,
    Cynder = 32,
    DebugMinion = 99,
    LegendaryBash = 404,
    LegendarySpyro = 416,
    LegendaryTriggerHappy = 419,
    LegendaryChopChop = 430,

    // Giants
    JetVac = 100,
    Swarm = 101,
    Crusher = 102,
    Flashwing = 103,
    HotHead = 104,
    HotDog = 105,
    Chill = 106,
    Thumpback = 107,
    PopFizz = 108,
    Ninjinni = 109,
    Bouncer = 110,
    Sprocket = 111,
    TreeRex = 112,
    Shroomboom = 113,
    EyeBrawl = 114,
    FrightRider = 115,

    // Minis
    Bop = 502,
    Spry = 503,
    Hijinx = 504,
    Terrabite = 505,
    Breeze = 506,
    Weerupter = 507,
    PetVac = 508,
    SmallFry = 509,
    Drobit = 510,
    GillRunt = 514,
    TriggerSnappy = 519,
    WhisperElf = 526,
    Barkley = 540,
    Thumpling = 541,
    MiniJini = 542,
    EyeSmall = 543,

    // Swap Force
    BoomJetBottom = 1000,
    FreeRangerBottom = 1001,
    RubbleRouserBottom = 1002,
    DoomStoneBottom = 1003,
    BlastZoneBottom = 1004,
    FireKrakenBottom = 1005,
    StinkBombBottom = 1006,
    GrillaDrillaBottom = 1007,
    FortuneBottom = 1008,
    TrapShadowBottom = 1009,
    MagnaChargeBottom = 1010,
    SpyRiseBottom = 1011,
    NightShiftBottom = 1012,
    RattleShakeBottom = 1013,
    FreezeBladeBottom = 1014,
    WashBucklerBottom = 1015,

    BoomJetTop = 2000,
    FreeRangerTop = 2001,
    RubbleRouserTop = 2002,
    DoomStoneTop = 2003,
    BlastZoneTop = 2004,
    FireKrakenTop = 2005,
    StinkBombTop = 2006,
    GrillaDrillaTop = 2007,
    FortuneTop = 2008,
    TrapShadowTop = 2009,
    MagnaChargeTop = 2010,
    SpyRiseTop = 2011,
    NightShiftTop = 2012,
    RattleShakeTop = 2013,
    FreezeBladeTop = 2014,
    WashBucklerTop = 2015,

    TemplateBottom = 1999,
    TemplateTop = 2999,
    
    Scratch = 3000,
    PopThorn = 3001,
    SlobberTooth = 3002,
    Scorp = 3003,
    Fryno = 3004,
    Smolderdash = 3005,
    BumbleBlast = 3006,
    ZooLou = 3007,
    DuneBug = 3008,
    StarStrike = 3009,
    Countdown = 3010,
    WindUp = 3011,
    RollerBrawl = 3012,
    GrimCreeper = 3013,
    RipTide = 3014,
    PunkShock = 3015,
    
    // Trap Team
    Gusto = 450,
    Thunderbolt = 451,
    FlingKong = 452,
    Blades = 453,
    Wallop = 454,
    HeadRush = 455,
    FistBump = 456,
    RockyRoll = 457,
    WildFire = 458,
    KaBoom = 459,
    TrailBlazer = 460,
    Torch = 461,
    SnapShot = 462,
    LobStar = 463,
    FlipWreck = 464,
    Echo = 465,
    Blastermind = 466,
    Enigma = 467,
    DejaVu = 468,
    CobraCadabra = 469,
    Jawbreaker = 470,
    Gearshift = 471,
    Chopper = 472,
    TreadHead = 473,
    Bushwhack = 474,
    TuffLuck = 475,
    FoodFight = 476,
    HighFive = 477,
    KryptKing = 478,
    ShortCut = 479,
    BatSpin = 480,
    FunnyBone = 481,
    KnightLight = 482,
    SpotLight = 483,
    KnightMare = 484,
    Blackout = 485,
    Template = 3999,

    // Superchargers
    Fiesta = 3400,
    HighVolt = 3401,
    Splat = 3402,
    Stormblade = 3406,
    SmashHit = 3411,
    Spitfire = 3412,
    DriverJetVac = 3413,
    DriverTriggerHappy = 3414,
    DriverStealthElf = 3415,
    DriverTerrafin = 3416,
    DriverRollerBrawl = 3417,
    DriverPopFizz = 3420,
    DriverEruptor = 3421,
    DriverGillGrunt = 3422,
    DonkeyKong = 3423,
    Bowser = 3424,
    DiveClops = 3425,
    Astroblast = 3426,
    Nightfall = 3427,
    Thrillipede = 3428,

    Fusion = 4500,
    Synergy = 4501,
    Unity = 4502,
    BlueFalcon = 4503,

    // Imaginators
    KingPen = 601,
    TriTip = 602,
    ChopScotch = 603,
    BoomBloom = 604,
    Pitboss = 605,
    Barbella = 606,
    AirStrike = 607,
    Ember = 608,
    Ambush = 609,
    DrKrankcase = 610,
    Hoodsickle = 611,
    TaeKwonCrow = 612,
    GoldenQueen = 613,
    Wolfgang = 614,
    PainYatta = 615,
    StarCast = 617,
    Buckshot = 618,
    Aurora = 619,
    FlareWolf = 620,
    ChompyMage = 621,
    BadJuju = 622,
    GraveClobber = 623,
    BlasterTron = 624,
    RoBow = 625,
    ChainReaction = 626,
    Kaos = 627,
    Wildstorm = 628,
    Tidepool = 629,
    CrashBandicoot = 630,
    DrNeoCortex = 631,

    ImaginatorTemplate = 699
}

#[repr(u16)]
#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq)]
pub enum Item {
    // Spyro's
    Anvil = 200,
    SecretStash = 201,
    Regeneration = 202,
    CrossedSwords = 203,
    Hourglass = 204,
    Shield = 205,
    SpeedBoots = 206,
    Sparx = 207,


    // Giants
    Cannon = 208,
    Catapult = 209,

    // Swap Force
    Hammer = 3200,
    Diamonds = 3201,
    GoldenSheep = 3202,
    Gramophone = 3203,
    HatPromo = 3204,

    // Trap Team
    HandOfFate = 230,
    PiggyBank = 231,
    RocketRam = 232,
    TikiSpeaky = 233,

    // Imaginators
    BlueChest = 235
}

#[repr(u16)]
#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq)]
pub enum ImaginatorCrystal {
    Magic = 680,
    Water = 681,
    Air = 682,
    Undead = 683,
    Tech = 684,
    Fire = 685,
    Earth = 686,
    Life = 687,
    Dark = 688,
    Light = 689,
}

#[repr(u16)]
#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq)]
pub enum Trap {
    Magic = 210,
    Water = 211,
    Air = 212,
    Death = 213,
    Tech = 214,
    Fire = 215,
    Earth = 216,
    Life = 217,
    Dark = 218,
    Light = 219,
    Kaos = 220,
}

#[repr(u16)]
#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq)]
pub enum Expansion {
    // Spyro's
    Dragon = 300,
    Ice = 301,
    Pirate = 302,
    Undead = 303,
    PVPUnlock = 304,

    // Swap Force
    SheepWreckIsland = 3300,
    ClockTower = 3301,
    BattleArena1 = 3302,
    BattleArena2 = 3303,

    // Trap Taeam
    MirrorOfMystery = 305,
    NightmareExpress = 306,
    SunscraperSpire = 307,
    MidnightMuseum = 308,

    // Superchargers
    SkyRacingPack = 3500,
    LandRacingPack = 3501,
    SeaRacingPack = 3502,
    KaosRacingPack = 3503,

    // Imaginators
    GryphonParkObservatory = 310,
    EnchantedElvenForest = 311,
}

#[repr(u16)]
#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq)]
pub enum Vehicle {
    Template = 4999,
    TemplateLand = 5999,
    TemplateAir = 6999,
    TemplateSea = 7999,

    Vehicle2015 = 3214,
    JetStream = 3220,
    TombBuggy = 3221,
    ReefRipper = 3222,
    BurnCycle = 3223,
    HotStreak = 3224,
    SharkTank = 3225,
    ThumpTruck = 3226,
    CryptCrusher = 3227,
    StealthStinger = 3228,
    DiveBomber = 3231,
    SkySlicer = 3232,
    ClownCruiser = 3233,
    GoldRusher = 3234,
    ShieldStriker = 3235,
    SunRunner = 3236,
    SeaShadow = 3237,
    SplatterSplasher = 3238,
    SodaSkimmer = 3239,
    BarrelBlaster = 3240,
    BuzzWing = 3241,
}

impl std::fmt::Display for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Into<u16> for Character {
    fn into(self) -> u16 {
        self as u16
    }
}

impl TryFrom<u16> for Character {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match CHARACTER_INDEX.get(&value) {
            Some(&c) => Ok(c),
            None => Err("Invalid Character value")
        }
    }
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Into<u16> for Item {
    fn into(self) -> u16 {
        self as u16
    }
}

impl TryFrom<u16> for Item {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match ITEM_INDEX.get(&value) {
            Some(&item) => Ok(item),
            None => Err("Invalid Item value")
        }
    }
}

impl std::fmt::Display for ImaginatorCrystal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Into<u16> for ImaginatorCrystal {
    fn into(self) -> u16 {
        self as u16
    }
}

impl TryFrom<u16> for ImaginatorCrystal {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match CRYSTAL_INDEX.get(&value) {
            Some(&crys) => Ok(crys),
            None => Err("Invalid Imaginator Crystal value")
        }
    }
}

impl std::fmt::Display for Trap {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Into<u16> for Trap {
    fn into(self) -> u16 {
        self as u16
    }
}

impl TryFrom<u16> for Trap {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match TRAP_INDEX.get(&value) {
            Some(&t) => Ok(t),
            None => Err("Invalid Trap value")
        }
    }
}

impl std::fmt::Display for Vehicle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Into<u16> for Vehicle {
    fn into(self) -> u16 {
        self as u16
    }
}

impl TryFrom<u16> for Vehicle {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match VEHICLE_INDEX.get(&value) {
            Some(&v) => Ok(v),
            None => Err("Invalid Vehicle value")
        }
    }
}

impl std::fmt::Display for Expansion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Into<u16> for Expansion {
    fn into(self) -> u16 {
        self as u16
    }
}

impl TryFrom<u16> for Expansion {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match EXPANSION_INDEX.get(&value) {
            Some(&exp) => Ok(exp),
            None => Err("Invalid Expansion value")
        }
    }
}
