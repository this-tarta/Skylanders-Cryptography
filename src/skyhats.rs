use std::{collections::HashMap, sync::LazyLock};
pub use strum::IntoEnumIterator;
use strum_macros::EnumIter;

static HAT_INDEX: LazyLock<HashMap<u16, Hat>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    for var in Hat::iter() {
        m.insert(var as u16, var);
    }
    m
});

#[repr(u16)]
#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq)]
pub enum Hat {
    // Spyro's
	None = 0,
	Army = 1,
	French = 2,
	Goggles = 3,
	Mining = 4,
	Officer = 5,
	Pirate = 6,
	Propeller = 7,
	Raccoon = 8,
	Straw = 9,
	Sunday = 10,
	Tophat = 11,
	Viking = 12,
	WorldWarI = 13,
	Anvil = 14,
	Beret = 15,
	BirthdayCap = 16,
	Bonehead = 17,
	Bowler = 18,
	Bunny = 19,
	Carmen = 20,
	Chef = 21,
	Cowboy = 22,
	CrazyHair = 23,
	Crown = 24,
	DevilHorns = 25,
	Eyeball = 26,
	Fez = 27,
	Halo = 28,
	Jester = 29,
	Mercury = 30,
	Moose = 31,
	Plunger = 32,
	Pot = 33,
	Rocket = 34,
	Santa = 35,
	Tiki = 36,
	Trojan = 37,
	Unicorn = 38,
	Wizard = 39,
	Pumpkin = 40,
	PirateBandana = 41,
	Cossack = 42,
	FlowerPot = 43,
	Balloon = 44,
	BirthdayCake = 45,
	
    // Giants
	VintageBaseballCap = 46,
	BowlingPin = 48,
	Officer2 = 49,
	FirefighterHelmet = 50,
	Graduation = 51,
	Lampshade = 52,
	Mariachi = 53,
	PaperFastFood = 55,
	Pilgrim = 56,
	PoliceSiren = 57,
	PurpleFedora = 58,
	Archer = 59,
	Safari = 61,
	Sailor = 62,
	Dancer = 64,
	TrafficCone = 65,
	Turban = 66,
	BattleHelmet = 67,
	BottleCap = 68,
	Carrot = 70,
	Elf = 72,
	Fishing = 73,
	Future = 74,
	Nefertiti = 75,
	Pants = 77,
	Princess = 78,
	ToySoldier = 79,
	Trucker = 80,
	Umbrella = 81,
	Showtime = 82,
	Caesar = 83,
	FlowerFairy = 84,
	Funnel = 85,
	Scrumshanks = 86,
	Biter = 87,
	Atom = 88,
	Sombrero = 89,
	Rasta = 90,
	Kufi = 91,
	KnightHelm = 92,
	DanglingCarrot = 93,
	BronzeTopHat = 94,
	SilverTopHat = 95,
	GoldTopHat = 96,
	
    // Swap Force
	Rain = 97,
	SnorkelingMask = 98,
	Greeble = 99,
	Volcano = 100,
	Boater = 101,
	Stone = 102,
	Stovepipe = 103,
	Boonie = 104,
	Sawblade = 105,
	SunBonnet = 106,
	Gaucho = 107,
	Roundlet = 108,
	Capuchon = 109,
	Tricorn = 110,
	FeatheredHeaddress = 111,
	BearskinCap = 112,
	Fishbone = 113,
	SkiCap = 114,
	CrownOfFrost = 115,
	FourWinds = 116,
	Beacon = 117,
	FlowerGarland = 118,
	TreeBranch = 119,
	AviatorsCap = 120,
	Asteroid = 121,
	Crystal = 122,
	CreepyHelm = 123,
	FancyRibbon = 124,
	DeelyBoppers = 125,
	BeanieCap = 126,
	Leprechaun = 127,
	Shark = 128,
	LifePreserver = 129,
	GlitteringTiara = 130,
	GreatHelm = 131,
	SpaceHelmet = 132,
	UFO = 133,
	WhirlwindDiadem = 134,
	ObsidianHelm = 135,
	Lilypad = 136,
	CrownOfFlames = 137,
	RunicHeadband = 138,
	Clockwork = 139,
	Cactus = 140,
	SkullHelm = 141,
	Gloop = 142,
	Puma = 143,
	Elephant = 144,
	TigerSkin = 145,
	TeethTop = 146,
	Turkey = 147,
	Afro = 148,
	BaconBandana = 149,
	AwesomeHat = 150,
	CardShark = 151,
	EasterTriggerHappy = 152,
	HolidayBumbleBlast = 153,
	SoccerCountdown = 154,
	
    // Trap Team
	Beetle = 155,
	Brain = 156,
	Brainiac = 157,
	Bucket = 158,
	Cactus02 = 159,
	Fan = 160,
	Chinese = 161,
	Clown = 162,
	ClownBowler = 163,
	Colander = 164,
	Kepi = 165,
	Cornucopia = 166,
	Cubano = 167,
	Cycling = 168,
	DaisyCrown = 169,
	Skull = 170,
	DundeeSlouch = 171,
	ElfMini = 172,
	Castro = 173,
	Garrison = 174,
	Gondolier = 175,
	Hunting = 176,
	Juicer = 177,
	Kokoshnik = 178,
	Medic = 179,
	Melon = 180,
	Mountie = 181,
	Nurse = 182,
	Palm = 183,
	Paperboy = 184,
	ParrotRoosting = 185,
	Projector = 186,
	Pot02 = 187,
	Radar = 188,
	Tiara = 189,
	RubberGlove = 190,
	Rugby = 191,
	SharkFin = 192,
	Sherlock = 193,
	Shower = 194,
	Bobby = 195,
	Hedgehog = 196,
	Steampunk = 197,
	Stewardess = 198,
	SundayFancy = 199,
	Tibet = 200,
	Trash = 201,
	Turtle = 202,
	UFO02 = 203,
	Vespa = 204,
	Volcano02 = 205,
	WaterBallet = 206,
	WilliamTell = 207,
	Zulu = 208,
	RudeBoyChecker = 209,
	PorkPieChecker = 210,
	AlarmClock = 211,
	Armadillo = 212,
	BearTrap = 213,
	Croissant = 214,
	WeatherVane = 215,
	Rainbow = 216,
	EyeOfKaos = 217,
	BatEars = 218,
	Lightbulb = 219,
	Fireflies = 220,
	ShadowGhost = 221,
	Lighthouse = 222,
	TinFoil = 223,
	NightCap = 224,
	Storm = 225,
	KnightmareGold = 226,
	Duck = 227,
	Pyramid = 228,
	Windmill = 229,
	Wizard02 = 230,
	Confectioner = 232,
	Eggshell = 233,
	Candle = 234,
	HelmDark = 235,
	Orbit = 236,
	Bellhop = 237,
	KnightmareBronze = 238,
	KnightmareSilver = 239,
	Raver = 240,
	Shire = 241,
	Mongol = 242,
	FishermanSlicker = 243,
	MedievalBard = 244,
	Wooden = 245,
	Carnival = 246,
	Coconut = 247,
	Wilikin = 248,
	Frostfest = 249,
	Molekin = 250,
	Sheepwrecked = 251,
	OldRuins = 252,
	Oracle = 253,
	Offset2015 = 256,
	
    // SuperChargers
	DiveBomber = 260,
	SeaShadow = 261,
	BurnCycle = 262,
	ReefRipper = 263,
	JetStream = 264,
	SodaSkimmer = 265,
	TombBuggy = 266,
	StealthStinger = 267,
	SharkTank = 268,
	GoldRusher = 269,
	SplatterSplasher = 270,
	ThumpTruck = 271,
	BuzzWing = 272,
	ShieldStriker = 273,
	SunRunner = 274,
	HotStreak = 275,
	SkySlicer = 276,
	CryptCrusher = 277,
	Mags = 278,
	KaosCrown = 279,
	EonHelm = 280,
}

impl std::fmt::Display for Hat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Into<u16> for Hat {
    fn into(self) -> u16 {
        self as u16
    }
}

impl TryFrom<u16> for Hat {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match HAT_INDEX.get(&value) {
            Some(&h) => Ok(h),
            None => Err("Invalid Hat value")
        }
    }
}