#![allow(unused)]

pub mod armor;
pub mod etc_item;
pub mod weapon;

use crate::backend::item::{ItemAdditionalInfoAction, ItemDropInfoAction};
use crate::backend::WindowParams;
use crate::data::{ItemId, ItemSetId, QuestId};
use crate::entity::item::weapon::Weapon;
use crate::entity::CommonEntity;
use crate::util::ASCF;
use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use strum_macros::{Display, EnumIter};

#[derive(Clone)]
pub struct Item {
    pub(crate) id: ItemId,
    pub(crate) name: String,
    pub(crate) desc: String,
}

impl<T: CommonEntity<ItemId, ()>> From<&T> for Item {
    fn from(val: &T) -> Self {
        Self {
            id: val.id(),
            name: val.name(),
            desc: val.desc(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct ItemBattleStats {
    pub(crate) p_defense: u16,
    pub(crate) m_defense: u16,
    pub(crate) p_attack: u16,
    pub(crate) m_attack: u16,
    pub(crate) p_attack_speed: u16,
    pub(crate) p_hit: f32,
    pub(crate) m_hit: f32,
    pub(crate) p_critical: f32,
    pub(crate) m_critical: f32,
    pub(crate) speed: u8,
    pub(crate) shield_defense: u16,
    pub(crate) shield_defense_rate: u8,
    pub(crate) p_avoid: f32,
    pub(crate) m_avoid: f32,
    pub(crate) property_params: u16,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct ItemDropMeshInfo {
    pub(crate) mesh: String,
    pub(crate) textures: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct ItemDropInfo {
    pub(crate) drop_type: DropType,
    pub(crate) drop_animation_type: DropAnimationType,
    pub(crate) drop_radius: u8,
    pub(crate) drop_height: u8,
    pub(crate) drop_mesh_info: Vec<ItemDropMeshInfo>,
    pub(crate) complete_item_drop_sound: String,
    pub(crate) drop_sound: String,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct ItemIcons {
    pub(crate) icon_1: String,
    pub(crate) icon_2: String,
    pub(crate) icon_3: String,
    pub(crate) icon_4: String,
    pub(crate) icon_5: String,
    pub(crate) icon_panel: String,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct ItemAdditionalInfo {
    pub(crate) has_animation: bool,
    pub(crate) include_items: Vec<ItemId>,
    pub(crate) max_energy: u32,
    pub(crate) look_change: String,
    pub(crate) hide_cloak: bool,
    pub(crate) unk: bool,
    pub(crate) hide_armor: bool,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct ItemBaseInfo {
    pub(crate) id: ItemId,
    pub(crate) name: String,
    pub(crate) additional_name: String,
    pub(crate) color: ItemNameColor,
    pub(crate) desc: String,
    pub(crate) tooltip_texture: String,

    pub(crate) popup: i16,
    pub(crate) use_order: u32,
    pub(crate) default_action: ItemDefaultAction,
    pub(crate) set_id: ItemSetId,

    pub(crate) is_trade: bool,
    pub(crate) is_drop: bool,
    pub(crate) is_destruct: bool,
    pub(crate) is_private_store: bool,
    pub(crate) is_npc_trade: bool,
    pub(crate) is_commission_store: bool,
    pub(crate) crystallizable: bool,

    pub(crate) keep_type: KeepType,
    pub(crate) inventory_type: InventoryType,
    pub(crate) material: ItemMaterial,
    pub(crate) body_part: BodyPart,
    pub(crate) quality: ItemQuality,
    pub(crate) crystal_type: CrystalType,

    pub(crate) durability: u16,
    pub(crate) weight: u16,
    pub(crate) default_price: i64,
    pub(crate) is_premium: bool,
    pub(crate) is_blessed: bool,
    pub(crate) property_params: i16,
    pub(crate) equip_sound: String,

    pub(crate) related_quests: Vec<QuestId>,

    pub(crate) icons: WindowParams<ItemIcons, (), (), ()>,
    pub(crate) additional_info: WindowParams<ItemAdditionalInfo, (), ItemAdditionalInfoAction, ()>,
    pub(crate) drop_info: WindowParams<ItemDropInfo, (), ItemDropInfoAction, ()>,
    pub(crate) battle_stats: WindowParams<ItemBattleStats, (), (), ()>,
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
pub enum InventoryType {
    #[default]
    Weapon = 1,
    Unk2 = 2,
    Unk3 = 3,
    Unk4 = 4,
    Unk5 = 5,
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
pub enum ItemMaterial {
    #[default]
    Unk0 = 0,
    Unk1 = 1,
    Unk2 = 2,
    Unk3 = 3,
    Unk4 = 4,
    Unk5 = 5,
    Unk6 = 6,
    Unk7 = 7,
    Unk8 = 8,
    Unk9 = 9,
    Unk10 = 10,
    Unk11 = 11,
    Unk12 = 12,
    Unk13 = 13,
    Unk14 = 14,
    Unk15 = 15,
    Unk16 = 16,
    Unk17 = 17,
    Unk18 = 18,
    Unk19 = 19,
    Unk20 = 20,
    Unk21 = 21,
    Unk22 = 22,
    Unk23 = 23,
    Unk24 = 24,
    Unk25 = 25,
    Unk26 = 26,
    Unk27 = 27,
    Unk28 = 28,
    Unk29 = 29,
    Unk30 = 30,
    Unk31 = 31,
    Unk32 = 32,
    Unk33 = 33,
    Unk34 = 34,
    Unk35 = 35,
    Unk36 = 36,
    Unk37 = 37,
    Unk38 = 38,
    Unk39 = 39,
    Unk40 = 40,
    Unk41 = 41,
    Unk42 = 42,
    Unk43 = 43,
    Unk44 = 44,
    Unk45 = 45,
    Unk46 = 46,
    Unk47 = 47,
    Unk48 = 48,
    Unk49 = 49,
    Unk50 = 50,
    Unk51 = 51,
    Unk52 = 52,
    Unk53 = 53,
    Unk54 = 54,
    Unk55 = 55,
    Unk56 = 56,
    Unk57 = 57,
    Unk58 = 58,
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
pub enum BodyPart {
    #[default]
    WolfWeapon = 0,
    Unk1 = 1,
    Unk2 = 2,
    Unk3 = 3,
    Unk4 = 4,
    Unk5 = 5,
    Unk6 = 6,
    TwoHandedWeapon = 7,
    Unk8 = 8,
    Unk9 = 9,
    Unk10 = 10,
    Unk11 = 11,
    Unk12 = 12,
    Unk13 = 13,
    Unk14 = 14,
    Unk15 = 15,
    Unk16 = 16,
    Unk17 = 17,
    Unk18 = 18,
    Unk19 = 19,
    Unk20 = 20,
    Unk21 = 21,
    Unk22 = 22,
    Unk23 = 23,
    Unk24 = 24,
    Unk25 = 25,
    Unk26 = 26,
    Unk27 = 27,
    Unk28 = 28,
    Unk29 = 29,
    Unk30 = 30,
    Unk31 = 31,
    Unk32 = 32,
    Unk33 = 33,
    OneHandedWeapon = 34,
    Shield = 35,
    None = 9999,
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
pub enum CrystalType {
    #[default]
    NG,
    D,
    C,
    B,
    A,
    S,
    S80,
    S84,
    R,
    R95,
    R99,
    NoRang,
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
pub enum DropType {
    #[default]
    Unk0,
    Unk1,
    Unk2,
    Unk3,
    Unk4,
    Unk5,
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
pub enum KeepType {
    #[default]
    Unk0,
    Unk1,
    Unk2,
    Unk3,
    Unk4,
    Unk5,
    Unk6,
    Unk7,
    Unk8,
    Unk9,
    Unk10,
    Unk11,
    Unk12,
    Unk13,
    Unk14,
    Unk15,
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
pub enum DropAnimationType {
    #[default]
    Unk0,
    Unk1,
    Unk2,
    Unk3,
    Unk4,
    Unk5,
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
pub enum ItemNameColor {
    #[default]
    Common,
    Normal,
    Rare,
    Epic,
    Blessed,
    Dragon,
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
pub enum ItemQuality {
    #[default]
    Common,
    Normal,
    Rare,
    Epic,
    Blessed,
    Dragon,
}

#[derive(Serialize, Deserialize, Debug, Default, EnumIter, Eq, PartialEq, Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum ItemDefaultAction {
    action_bless_spiritshot,
    action_calc,
    action_call_skill,
    action_capsule,
    action_create_mpcc,
    action_dice,
    action_equip,
    action_fishingshot,
    action_harvest,
    action_hide_name,
    action_keep_exp,
    action_nick_color,
    #[default]
    action_none,
    action_peel,
    action_recipe,
    action_seed,
    action_show_adventurer_guide_book,
    action_show_html,
    action_show_ssq_status,
    action_show_tutorial,
    action_skill_maintain,
    action_skill_reduce,
    action_skill_reduce_on_skill_success,
    action_soulshot,
    action_spiritshot,
    action_start_quest,
    action_summon_soulshot,
    action_summon_spiritshot,
    action_xmas_open,
}

impl ItemDefaultAction {
    pub fn label_text(&self) -> String {
        match self {
            ItemDefaultAction::action_bless_spiritshot => "Bless Spiritshot".to_string(),
            ItemDefaultAction::action_calc => "Calc".to_string(),
            ItemDefaultAction::action_call_skill => "Call Skill".to_string(),
            ItemDefaultAction::action_capsule => "Capsule".to_string(),
            ItemDefaultAction::action_create_mpcc => "Create Mpcc".to_string(),
            ItemDefaultAction::action_dice => "Dice".to_string(),
            ItemDefaultAction::action_equip => "Equip".to_string(),
            ItemDefaultAction::action_fishingshot => "Fishing Shot".to_string(),
            ItemDefaultAction::action_harvest => "Harvest".to_string(),
            ItemDefaultAction::action_hide_name => "Hide Name".to_string(),
            ItemDefaultAction::action_keep_exp => "Keep Exp".to_string(),
            ItemDefaultAction::action_nick_color => "Nick Color".to_string(),
            ItemDefaultAction::action_none => "None".to_string(),
            ItemDefaultAction::action_peel => "Peel".to_string(),
            ItemDefaultAction::action_recipe => "Recipe".to_string(),
            ItemDefaultAction::action_seed => "Seed".to_string(),
            ItemDefaultAction::action_show_adventurer_guide_book => {
                "Show Adventurer Guide Book".to_string()
            }
            ItemDefaultAction::action_show_html => "Show Html".to_string(),
            ItemDefaultAction::action_show_ssq_status => "Show Ssq Status".to_string(),
            ItemDefaultAction::action_show_tutorial => "Show Tutorial".to_string(),
            ItemDefaultAction::action_skill_maintain => "Skill Maintain".to_string(),
            ItemDefaultAction::action_skill_reduce => "Skill Reduce".to_string(),
            ItemDefaultAction::action_skill_reduce_on_skill_success => {
                "Skill Reduce on Skill Success".to_string()
            }
            ItemDefaultAction::action_soulshot => "Soulshot".to_string(),
            ItemDefaultAction::action_spiritshot => "Spiritshot".to_string(),
            ItemDefaultAction::action_start_quest => "Start Quest".to_string(),
            ItemDefaultAction::action_summon_soulshot => "Summon Soulshot".to_string(),
            ItemDefaultAction::action_summon_spiritshot => "Summon Spiritshot".to_string(),
            ItemDefaultAction::action_xmas_open => "Xmas Open".to_string(),
        }
    }
}

impl Display for ItemDefaultAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ItemDefaultAction::action_bless_spiritshot => "action_bless_spiritshot\0".to_string(),
            ItemDefaultAction::action_calc => "action_calc\0".to_string(),
            ItemDefaultAction::action_call_skill => "action_call_skill\0".to_string(),
            ItemDefaultAction::action_capsule => "action_capsule\0".to_string(),
            ItemDefaultAction::action_create_mpcc => "action_create_mpcc\0".to_string(),
            ItemDefaultAction::action_dice => "action_dice\0".to_string(),
            ItemDefaultAction::action_equip => "action_equip\0".to_string(),
            ItemDefaultAction::action_fishingshot => "action_fishingshot\0".to_string(),
            ItemDefaultAction::action_harvest => "action_harvest\0".to_string(),
            ItemDefaultAction::action_hide_name => "action_hide_name\0".to_string(),
            ItemDefaultAction::action_keep_exp => "action_keep_exp\0".to_string(),
            ItemDefaultAction::action_nick_color => "action_nick_color\0".to_string(),
            ItemDefaultAction::action_none => "action_none\0".to_string(),
            ItemDefaultAction::action_peel => "action_peel\0".to_string(),
            ItemDefaultAction::action_recipe => "action_recipe\0".to_string(),
            ItemDefaultAction::action_seed => "action_seed\0".to_string(),
            ItemDefaultAction::action_show_adventurer_guide_book => {
                "action_show_adventurer_guide_book\0".to_string()
            }
            ItemDefaultAction::action_show_html => "action_show_html\0".to_string(),
            ItemDefaultAction::action_show_ssq_status => "action_show_ssq_status\0".to_string(),
            ItemDefaultAction::action_show_tutorial => "action_show_tutorial\0".to_string(),
            ItemDefaultAction::action_skill_maintain => "action_skill_maintain\0".to_string(),
            ItemDefaultAction::action_skill_reduce => "action_skill_reduce\0".to_string(),
            ItemDefaultAction::action_skill_reduce_on_skill_success => {
                "action_skill_reduce_on_skill_success\0".to_string()
            }
            ItemDefaultAction::action_soulshot => "action_soulshot\0".to_string(),
            ItemDefaultAction::action_spiritshot => "action_spiritshot\0".to_string(),
            ItemDefaultAction::action_start_quest => "action_start_quest\0".to_string(),
            ItemDefaultAction::action_summon_soulshot => "action_summon_soulshot\0".to_string(),
            ItemDefaultAction::action_summon_spiritshot => "action_summon_spiritshot\0".to_string(),
            ItemDefaultAction::action_xmas_open => "action_xmas_open\0".to_string(),
        };
        write!(f, "{}", str)
    }
}

impl ItemDefaultAction {
    pub fn from_ascf(value: &ASCF) -> Self {
        match &*value.0 {
            "action_bless_spiritshot\0" => Self::action_bless_spiritshot,
            "action_calc\0" => Self::action_calc,
            "action_call_skill\0" => Self::action_call_skill,
            "action_capsule\0" => Self::action_capsule,
            "action_create_mpcc\0" => Self::action_create_mpcc,
            "action_dice\0" => Self::action_dice,
            "action_equip\0" => Self::action_equip,
            "action_fishingshot\0" => Self::action_fishingshot,
            "action_harvest\0" => Self::action_harvest,
            "action_hide_name\0" => Self::action_hide_name,
            "action_keep_exp\0" => Self::action_keep_exp,
            "action_nick_color\0" => Self::action_nick_color,
            "action_none\0" => Self::action_none,
            "action_peel\0" => Self::action_peel,
            "action_recipe\0" => Self::action_recipe,
            "action_seed\0" => Self::action_seed,
            "action_show_adventurer_guide_book\0" => Self::action_show_adventurer_guide_book,
            "action_show_html\0" => Self::action_show_html,
            "action_show_ssq_status\0" => Self::action_show_ssq_status,
            "action_show_tutorial\0" => Self::action_show_tutorial,
            "action_skill_maintain\0" => Self::action_skill_maintain,
            "action_skill_reduce\0" => Self::action_skill_reduce,
            "action_skill_reduce_on_skill_success\0" => Self::action_skill_reduce_on_skill_success,
            "action_soulshot\0" => Self::action_soulshot,
            "action_spiritshot\0" => Self::action_spiritshot,
            "action_start_quest\0" => Self::action_start_quest,
            "action_summon_soulshot\0" => Self::action_summon_soulshot,
            "action_summon_spiritshot\0" => Self::action_summon_spiritshot,
            "action_xmas_open\0" => Self::action_xmas_open,

            _ => unreachable!(),
        }
    }
}
