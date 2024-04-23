use crate::data::Position;
use crate::entity::item::{ItemBaseInfo, ItemBattleStats};
use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use strum_macros::{Display, EnumIter};

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct Weapon {
    pub(crate) base_info: ItemBaseInfo,
    pub(crate) weapon_type: WeaponType,
    pub(crate) character_animation_type: CharacterAnimationType,
    pub(crate) battle_stats: ItemBattleStats,
    pub(crate) random_damage: RandomDamage,
    pub(crate) ertheia_fists_scale: f32,
    pub(crate) mesh_info: Vec<WeaponMeshInfo>,
    pub(crate) sound: Vec<String>,
    pub(crate) effect: String,
    pub(crate) mp_consume: WeaponMpConsume,
    pub(crate) soulshot_count: u8,
    pub(crate) spiritshot_count: u8,
    pub(crate) curvature: i16,
    pub(crate) unk: bool,
    pub(crate) can_equip_hero: bool,
    pub(crate) is_magic_weapon: bool,
    pub(crate) enchant_junk: i16,
    pub(crate) enchant_info: Vec<WeaponEnchantInfo>,
    pub(crate) variation_info: WeaponVariationInfo,
    pub(crate) can_ensoul: bool,
    pub(crate) ensoul_count: u8,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct WeaponVariationInfo {
    pub(crate) icon: Vec<String>,
    pub(crate) effect_1: u8,
    pub(crate) effect_2: u8,
    pub(crate) effect_3: u8,
    pub(crate) effect_4: u8,
    pub(crate) effect_5: u8,
    pub(crate) effect_6: u8,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct WeaponMeshInfo {
    pub(crate) mesh: String,
    pub(crate) unk: u8,
    pub(crate) texture: String,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct WeaponEnchantInfo {
    pub(crate) effect: String,
    pub(crate) effect_offset: Position,
    pub(crate) effect_scale: f32,
    pub(crate) effect_velocity: f32,

    pub(crate) mesh_offset: Position,
    pub(crate) mesh_scale: Position,

    pub(crate) particle_offset: Position,
    pub(crate) particle_scale: f32,

    pub(crate) ring_offset: Position,
    pub(crate) ring_scale: Position,
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
pub enum WeaponMpConsume {
    #[default]
    Unk0,
    Unk1,
    Unk2,
    Unk3,
    Unk4,
    Unk5,
    Unk6,
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
pub enum HandType {
    #[default]
    OneHand = 1,
    TwoHand = 2,
    DualSword = 3,
    Pole = 4,
    Bow = 5,
    Fists = 7,
    CrossBow = 8,
    Rapier = 9,
    TwoHandMagicBlunt = 14,
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
pub enum WeaponType {
    Shield = 0,
    #[default]
    Sword = 1,
    Blunt = 2,
    Dagger = 3,
    Pole = 4,
    Fists = 5,
    Bow = 6,
    Etc = 7,
    DualSword = 8,
    Rod = 10,
    Rapier = 11,
    CrossBow = 12,
    AncientSword = 13,
    DualDagger = 15,
    CrossBow2 = 17,
    DualBlunt = 18,
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
pub enum CharacterAnimationType {
    Shield = 0,
    #[default]
    OneHandedSword = 1,
    TwoHandedSword = 2,
    DualSword = 3,
    Spear = 4,
    Bow = 5,
    Dagger = 6,
    Fists = 7,
    CrossBow = 8,
    Rapier = 9,
    DualDagger = 10,
    CrossBow2 = 11,
    Dagger2 = 12,
    DualBlunt = 13,
    Staff = 14,
}

#[derive(
    Serialize,
    Deserialize,
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
pub enum RandomDamage {
    #[default]
    Zero = 0,
    One = 1,
    Five = 5,
    Ten = 10,
    Fifteen = 15,
    Twenty = 20,
    Forty = 40,
}

impl Display for RandomDamage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RandomDamage::Zero => write!(f, "0%"),
            RandomDamage::One => write!(f, "1%"),
            RandomDamage::Five => write!(f, "5%"),
            RandomDamage::Ten => write!(f, "10%"),
            RandomDamage::Fifteen => write!(f, "15%"),
            RandomDamage::Twenty => write!(f, "20%"),
            RandomDamage::Forty => write!(f, "40%"),
        }
    }
}
