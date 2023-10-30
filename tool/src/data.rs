use strum_macros::{Display, EnumIter};
const ADENA_ID: u32 = 57;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default, PartialOrd, Ord)]
pub struct ItemId(pub u32);

impl ItemId {
    pub fn is_adena(&self) -> bool {
        return self.0 == ADENA_ID;
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default, PartialOrd, Ord)]
pub struct QuestId(pub u32);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default, PartialOrd, Ord)]
pub struct NpcId(pub u32);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default, PartialOrd, Ord)]
pub struct HuntingZoneId(pub u32);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default, PartialOrd, Ord)]
pub struct SearchZoneId(pub u32);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default, PartialOrd, Ord)]
pub struct InstantZoneId(pub u32);

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Location {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Display, Debug, EnumIter, Eq, PartialEq, Copy, Clone)]
pub enum PlayerClass {
    Fighter = 0,
    Warrior = 1,
    Gladiator = 2,
    Warlord = 3,
    Knight = 4,
    Paladin = 5,
    DarkAvenger = 6,
    Rogue = 7,
    TreasureHunter = 8,
    Hawkeye = 9,
    Mage = 10,
    Wizard = 11,
    Sorceror = 12,
    Necromancer = 13,
    Warlock = 14,
    Cleric = 15,
    Bishop = 16,
    Prophet = 17,
    ElvenFighter = 18,
    ElvenKnight = 19,
    TempleKnight = 20,
    SwordSinger = 21,
    ElvenScout = 22,
    PlainsWalker = 23,
    SilverRanger = 24,
    ElvenMage = 25,
    ElvenWizard = 26,
    Spellsinger = 27,
    ElementalSummoner = 28,
    Oracle = 29,
    Elder = 30,
    DarkFighter = 31,
    PalusKnight = 32,
    ShillienKnight = 33,
    Bladedancer = 34,
    Assassin = 35,
    AbyssWalker = 36,
    PhantomRanger = 37,
    DarkMage = 38,
    DarkWizard = 39,
    Spellhowler = 40,
    PhantomSummoner = 41,
    ShillienOracle = 42,
    ShillienElder = 43,
    OrcFighter = 44,
    OrcRaider = 45,
    Destroyer = 46,
    OrcMonk = 47,
    Tyrant = 48,
    OrcMage = 49,
    OrcShaman = 50,
    Overlord = 51,
    Warcryer = 52,
    DwarvenFighter = 53,
    Scavenger = 54,
    BountyHunter = 55,
    Artisan = 56,
    Warsmith = 57,
    Duelist = 88,
    Dreadnought = 89,
    PhoenixKnight = 90,
    HellKnight = 91,
    Sagittarius = 92,
    Adventurer = 93,
    Archmage = 94,
    Soultaker = 95,
    ArcanaLord = 96,
    Cardinal = 97,
    Hierophant = 98,
    EvaTemplar = 99,
    SwordMuse = 100,
    WindRider = 101,
    MoonlightSentinel = 102,
    MysticMuse = 103,
    ElementalMaster = 104,
    EvaSaint = 105,
    ShillienTemplar = 106,
    SpectralDancer = 107,
    GhostHunter = 108,
    GhostSentinel = 109,
    StormScreamer = 110,
    SpectralMaster = 111,
    ShillienSaint = 112,
    Titan = 113,
    GrandKhauatari = 114,
    Dominator = 115,
    Doomcryer = 116,
    FortuneSeeker = 117,
    Maestro = 118,
    KamaelSoldierMale = 123,
    KamaelSoldierFemale = 124,
    Trooper = 125,
    Warder = 126,
    Berserker = 127,
    SoulBreakerMale = 128,
    SoulBreakerFemale = 129,
    Arbalester = 130,
    Doombringer = 131,
    SoulHoundMale = 132,
    SoulHoundFemale = 133,
    Trickster = 134,
    Inspector = 135,
    Judicator = 136,
}
