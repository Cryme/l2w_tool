use crate::backend::{SkillEnchantAction, SkillEnchantEditWindowParams, WindowParams};
use crate::data::{SkillId, VisualEffectId};
use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};

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

pub enum SkillAnimation {}

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
    pub animations: Vec<String>,
    pub visual_effect: VisualEffectId,
    pub icon: String,
    pub icon_panel: String,
    pub cast_bar_text_is_red: bool,
    pub rumble_self: u8,
    pub rumble_target: u8,
    pub skill_levels: Vec<SkillLevelInfo>,
    pub is_debuff: bool,
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
    pub effect_point: u32,
    pub available_enchants:
        Vec<WindowParams<EnchantInfo, (), SkillEnchantAction, SkillEnchantEditWindowParams>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct EnchantInfo {
    pub enchant_name: String,
    pub enchant_type: u32,
    pub skill_description: String,
    pub enchant_description: String,
    pub is_debuff: bool,
    pub enchant_levels: Vec<EnchantLevelInfo>,
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
    pub effect_point: u32,
}
