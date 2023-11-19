#![allow(clippy::upper_case_acronyms)]
use crate::backend::{SkillEnchantAction, SkillEnchantEditWindowParams, WindowParams};
use crate::data::{ItemId, SkillId, VisualEffectId};
use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};

#[derive(
    Serialize,
    Deserialize,
    Display,
    Debug,
    EnumIter,
    Eq,
    PartialEq,
    Copy,
    Clone,
    FromPrimitive,
    ToPrimitive,
)]
pub enum SkillType {
    ///Аквтивные вкладка Физические Умения
    Physical,
    ///Аквтивные вкладка Магические Умения
    Magical,
    ///Аквтивные вкладка Усиливающие Умения
    Buff,
    ///Аквтивные вкладка Ослабляющие Умения
    Debuff,
    ///Аквтивные вкладка Умения героя/клана/наставничества
    ClanActive,
    ///Активная вкладка Предметные Умения
    ItemActive,
    ///Аквтивные вкладка Включаемые
    Toggle,
    ///Аквтивные вкладка Перевоплощения
    Transformation,
    ///Аквтивные вкладка Включаемые, не отличает от Toggle
    AlsoToggle,
    ///Пассивная вкладка Умения Экипировки (Sword Mastery, Heavy Armor Mastery)
    EquipmentPassive = 11,
    ///Пассивная вкладка Спобности
    Abilities = 12,
    ///Пассивная вкладка Рассовые
    Race = 13,
    ///Пассивные вкладка Дополнительные Умения
    Additional = 14,
    ///Пассивные вкладка Умения героя/клана/наставничества
    ClanPassive = 15,
    ///Пассивные вкладка Предметные Умения
    ItemPassive = 16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Skill {
    pub id: SkillId,
    pub name: String,
    pub description: String,
    pub skill_type: SkillType,
    pub resist_cast: u8,
    pub magic_type: u8,
    pub cast_style: u8,
    pub skill_magic_type: u8,
    pub origin_skill: SkillId,
    pub is_double: bool,
    pub animations: Vec<SkillAnimation>,
    pub visual_effect: VisualEffectId,
    pub icon: String,
    pub icon_panel: String,
    pub cast_bar_text_is_red: bool,
    pub rumble_self: u8,
    pub rumble_target: u8,
    pub skill_levels: Vec<SkillLevelInfo>,
    pub is_debuff: bool,
    pub sound_info: WindowParams<SkillSoundInfo, (), (), ()>,
    pub use_condition: Option<WindowParams<SkillUseCondition, (), (), ()>>,
}

#[derive(
    Serialize,
    Deserialize,
    Display,
    Debug,
    Default,
    EnumIter,
    Eq,
    PartialEq,
    Copy,
    Clone,
    FromPrimitive,
    ToPrimitive,
)]
pub enum StatConditionType {
    #[default]
    None,
    HP,
    MP,
    CP,
    TargetHP,
}

#[derive(
    Serialize,
    Deserialize,
    Display,
    Debug,
    Default,
    EnumIter,
    Eq,
    PartialEq,
    Copy,
    Clone,
    FromPrimitive,
    ToPrimitive,
)]
pub enum EquipStatus {
    #[default]
    None,
    Shield,
    Weapon,
}

#[derive(
    Serialize,
    Deserialize,
    Display,
    Debug,
    Default,
    EnumIter,
    Eq,
    PartialEq,
    Copy,
    Clone,
    FromPrimitive,
    ToPrimitive,
)]
pub enum StatComparisonType {
    #[default]
    Lower,
    Higher,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct SkillUseCondition {
    pub(crate) equipment_condition: EquipStatus,
    pub(crate) weapon_types: Vec<u8>,
    pub(crate) stat_condition_type: StatConditionType,
    pub(crate) stat_percentage: u8,
    pub(crate) comparison_type: StatComparisonType,
    pub(crate) consumable_item_id: ItemId,
    pub(crate) item_count: u16,
    pub(crate) caster_prior_skill: Vec<PriorSkill>,
    pub(crate) target_prior_skill: Vec<PriorSkill>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct PriorSkill {
    pub id: SkillId,
    pub level: u32,
}

impl Skill {
    pub fn new(id: u32) -> Self {
        Self {
            id: SkillId(id),
            name: "".to_string(),
            description: "".to_string(),
            skill_type: SkillType::Physical,
            resist_cast: 0,
            magic_type: 0,
            cast_style: 0,
            skill_magic_type: 0,
            origin_skill: Default::default(),
            is_double: false,
            animations: vec![],
            visual_effect: Default::default(),
            icon: "".to_string(),
            icon_panel: "".to_string(),
            cast_bar_text_is_red: false,
            rumble_self: 0,
            rumble_target: 0,
            skill_levels: vec![],
            is_debuff: false,
            sound_info: WindowParams {
                inner: SkillSoundInfo::default(),
                opened: false,
                original_id: (),
                action: (),
                params: (),
            },

            use_condition: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SkillLevelInfo {
    pub level: u32,
    pub description_params: String,
    pub mp_cost: i16,
    pub hp_cost: i16,
    pub cast_range: u32,
    pub hit_time: f32,
    pub cool_time: f32,
    pub reuse_delay: f32,
    pub effect_point: i32,
    pub icon: Option<String>,
    pub icon_panel: Option<String>,
    pub description: Option<String>,
    pub available_enchants:
        Vec<WindowParams<EnchantInfo, (), SkillEnchantAction, SkillEnchantEditWindowParams>>,
}

impl Default for SkillLevelInfo {
    fn default() -> Self {
        Self {
            level: 1,
            description_params: "".to_string(),
            mp_cost: 0,
            hp_cost: 0,
            cast_range: 0,
            hit_time: 0.0,
            cool_time: 0.0,
            reuse_delay: 0.0,
            effect_point: 0,
            icon: None,
            icon_panel: None,
            description: None,
            available_enchants: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct SoundInfo {
    pub sound: String,
    pub vol: f32,
    pub rad: f32,
    pub delay: f32,
    pub source: u32,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct RacesSkillSoundInfo {
    pub mfighter: String,
    pub ffighter: String,
    pub mmagic: String,
    pub fmagic: String,
    pub melf: String,
    pub felf: String,
    pub mdark_elf: String,
    pub fdark_elf: String,
    pub mdwarf: String,
    pub fdwarf: String,
    pub morc: String,
    pub forc: String,
    pub mshaman: String,
    pub fshaman: String,
    pub mkamael: String,
    pub fkamael: String,
    pub mertheia: String,
    pub fertheia: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct SkillSoundInfo {
    pub spell_effect_1: SoundInfo,
    pub spell_effect_2: SoundInfo,
    pub spell_effect_3: SoundInfo,

    pub shot_effect_1: SoundInfo,
    pub shot_effect_2: SoundInfo,
    pub shot_effect_3: SoundInfo,

    pub exp_effect_1: SoundInfo,
    pub exp_effect_2: SoundInfo,
    pub exp_effect_3: SoundInfo,

    pub sound_before_cast: RacesSkillSoundInfo,
    pub sound_after_cast: RacesSkillSoundInfo,

    pub mextra_throw: String,
    pub fextra_throw: String,

    pub vol: f32,
    pub rad: f32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct EnchantInfo {
    pub enchant_name: String,
    pub enchant_icon: String,
    pub enchant_type: u32,
    pub skill_description: Option<String>,
    pub enchant_description: String,
    pub is_debuff: bool,
    pub enchant_levels: Vec<EnchantLevelInfo>,
}

impl Default for EnchantInfo {
    fn default() -> Self {
        Self {
            enchant_name: "New Enchant".to_string(),
            enchant_icon: "None".to_string(),
            enchant_type: 0,
            skill_description: None,
            enchant_description: "".to_string(),
            is_debuff: false,
            enchant_levels: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct EnchantLevelInfo {
    pub level: u32,
    pub skill_description_params: String,
    pub enchant_name_params: String,
    pub enchant_description_params: String,
    pub mp_cost: i16,
    pub hp_cost: i16,
    pub cast_range: u32,
    pub hit_time: f32,
    pub cool_time: f32,
    pub reuse_delay: f32,
    pub effect_point: i32,
    pub icon: Option<String>,
    pub icon_panel: Option<String>,
}

impl Default for EnchantLevelInfo {
    fn default() -> Self {
        Self {
            level: 1,
            skill_description_params: "".to_string(),
            enchant_name_params: "".to_string(),
            enchant_description_params: "".to_string(),
            mp_cost: 0,
            hp_cost: 0,
            cast_range: 0,
            hit_time: 0.0,
            cool_time: 0.0,
            reuse_delay: 0.0,
            effect_point: 0,
            icon: None,
            icon_panel: None,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(
    Serialize, Deserialize, Display, Debug, EnumIter, Eq, PartialEq, Copy, Clone, EnumString,
)]
pub enum SkillAnimation {
    ACT04,
    ACT08,
    ACT10,
    ACT12,
    AIRHIT_A,
    AIR_BIND,
    AWAKEN,
    B,
    BB01,
    BB02,
    C2,
    CACT03,
    CACT04,
    CACT05,
    CACT06,
    CACT07,
    CACT08,
    CACT09,
    CACT10,
    CACT11,
    CACT12,
    CACT13,
    CACT14,
    CACT15,
    CACT16,
    CACT17,
    CACT18,
    CACT19,
    CACT20,
    CACT22,
    CACT23,
    CACT24,
    CACT25,
    CACT26,
    CAL01,
    CAL02,
    CAL03,
    CAL08,
    CAL09,
    CAL10,
    CAL11,
    CAL12,
    CAL13,
    CAL14,
    CAL16,
    CAL18,
    CAL20,
    CAL21,
    CAL22,
    CAL23,
    CAL24,
    CALS01,
    CALS03,
    CALS04,
    CALS06,
    CALS10,
    CALS15,
    CALS16,
    CALS24,
    CAS01,
    CAS03,
    CAS07,
    CAS08,
    CAS09,
    CAS10,
    CAS14,
    CAS16,
    CAS20,
    CAS21,
    CAS22,
    CSPL01,
    CSPL02,
    CSPM01,
    CSPS01,
    D,
    D2,
    DD,
    DHIT,
    DI,
    DOWN_HAND,
    DRAG,
    DRAGQ,
    DUALCASTDRAG,
    DUALCASTSHOT,
    E2,
    ESP01,
    ESP02,
    ESP03,
    ESP04,
    ESP05,
    ESP06,
    ESP07,
    ESP08,
    ESP09,
    ESP10,
    ESP11,
    ESP12,
    ESP13,
    ESP14,
    ESP15,
    ESP16,
    ESP17,
    ESP18,
    ESP19,
    ESP21,
    EX01,
    EX02,
    EX03,
    EX04,
    EX05,
    EX07,
    EX08,
    EX09,
    EX11,
    EX12,
    EX13,
    EX14,
    EX15,
    EX17,
    EX20,
    EX22,
    EX23,
    EX24,
    EX25,
    EX26,
    EX27,
    EX29,
    EX30,
    EX31,
    EX32,
    EX34,
    EX35,
    EX36,
    EX37,
    EX38,
    EX39,
    EX40,
    EX41,
    EX43,
    EX44,
    F2,
    G2,
    H2,
    J,
    K,
    K2,
    KNOCK_HAND,
    L2,

    #[strum(serialize = "L2DAY - A", serialize = "A")]
    L2DAY_A,
    #[strum(serialize = "L2DAY - C", serialize = "C")]
    L2DAY_C,
    #[strum(serialize = "L2DAY - E", serialize = "E")]
    L2DAY_E,
    #[strum(serialize = "L2DAY - F", serialize = "F")]
    L2DAY_F,
    #[strum(serialize = "L2DAY - G", serialize = "G")]
    L2DAY_G,
    #[strum(serialize = "L2DAY - H", serialize = "H")]
    L2DAY_H,
    #[strum(serialize = "L2DAY - I", serialize = "I")]
    L2DAY_I,
    #[strum(serialize = "L2DAY - L", serialize = "L")]
    L2DAY_L,
    #[strum(serialize = "L2DAY - N", serialize = "N")]
    L2DAY_N,
    #[strum(serialize = "L2DAY - O", serialize = "O")]
    L2DAY_O,
    #[strum(serialize = "L2DAY - R", serialize = "R")]
    L2DAY_R,
    #[strum(serialize = "L2DAY - S", serialize = "S")]
    L2DAY_S,
    #[strum(serialize = "L2DAY - T", serialize = "T")]
    L2DAY_T,

    LEAP,
    M,
    MA01,
    MA02,
    MA03,
    MDD,
    MIX01,
    MIX02,
    MIX03,
    MIX04,
    MIX05,
    MIX06,
    MIX07,
    MIX08,
    MIX09,
    MIX51,
    MIX52,
    MIX53,
    MIX54,
    MIX55,
    MIX57,
    MIX58,
    MIX59,
    MIX60,
    MIX_A,
    MP01,
    MP02,
    MP03,
    MP04,
    MP05,
    MP06,
    MP07,
    MP08,
    MP10,
    MS01,
    MWAIT,
    M_SHOT_A,
    NONE,
    OPEN,
    P,
    PUSH,
    Q,
    SC_ATKWAIT,
    SC_DEATH,
    SMB01,
    SMB02,
    SMB03,
    SMC01,
    SMC03,
    SMD01,
    SME01,
    SME03,
    SPC01,
    SPC02,
    SPC03,
    SPC04,
    SPC05,
    U,
    V,
    W,
    X,
    Y,
    Z,
}
