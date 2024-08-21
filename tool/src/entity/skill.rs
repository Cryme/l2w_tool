#![allow(clippy::upper_case_acronyms)]
use crate::backend::entity_editor::WindowParams;
use crate::backend::entity_impl::skill::{
    SkillEditWindowParams, SkillEnchantAction, SkillEnchantEditWindowParams,
    SkillUceConditionAction,
};
use crate::data::{ItemId, SkillId};
use crate::entity::{CommonEntity, GetEditParams};
use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use strum_macros::{Display, EnumIter};

impl GetEditParams<SkillEditWindowParams> for Skill {
    fn edit_params(&self) -> SkillEditWindowParams {
        SkillEditWindowParams {
            current_level_index: self.skill_levels.len() - 1,
        }
    }
}

impl CommonEntity<SkillId> for Skill {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn desc(&self) -> String {
        self.description.clone()
    }

    fn id(&self) -> SkillId {
        self.id
    }

    fn changed(&self) -> bool {
        self._changed
    }

    fn deleted(&self) -> bool {
        self._deleted
    }

    fn new(id: SkillId) -> Self {
        Self {
            id,
            name: "New Skill".to_string(),
            description: "".to_string(),
            skill_type: SkillType::Physical,
            resist_cast: 0,
            magic_type: 0,
            cast_style: 0,
            skill_magic_type: 0,
            origin_skill: Default::default(),
            is_double: false,
            animations: vec!["None".to_string()],
            visual_effect: Default::default(),
            icon: "".to_string(),
            icon_panel: "".to_string(),
            cast_bar_text_is_red: false,
            rumble_self: 0,
            rumble_target: 0,
            skill_levels: vec![SkillLevelInfo::default()],
            is_debuff: false,
            sound_info: WindowParams {
                inner: SkillSoundInfo::default(),
                opened: false,
                initial_id: (),
                action: RwLock::new(()),
                params: (),
            },

            use_condition: None,

            _changed: false,
            _deleted: false,
        }
    }
}

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
    Default,
)]
pub enum SkillType {
    ///Аквтивные вкладка Физические Умения
    #[default]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
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
    pub animations: Vec<String>,
    pub visual_effect: String,
    pub icon: String,
    pub icon_panel: String,
    pub cast_bar_text_is_red: bool,
    pub rumble_self: u8,
    pub rumble_target: u8,
    pub skill_levels: Vec<SkillLevelInfo>,
    pub is_debuff: bool,
    pub sound_info: WindowParams<SkillSoundInfo, (), (), ()>,
    pub use_condition: Option<WindowParams<SkillUseCondition, (), SkillUceConditionAction, ()>>,

    #[serde(skip)]
    pub _changed: bool,
    #[serde(skip)]
    pub _deleted: bool,
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

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
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

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct PriorSkill {
    pub id: SkillId,
    pub level: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
    pub name: Option<String>,
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
            name: None,
            description: None,
            available_enchants: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct SoundInfo {
    pub sound: String,
    pub vol: f32,
    pub rad: f32,
    pub delay: f32,
    pub source: u32,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
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

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
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
