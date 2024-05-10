use crate::backend::entity_editor::WindowParams;
use crate::backend::entity_impl::item::weapon::{WeaponEnchantAction, WeaponVariationAction};
use crate::data::{ItemId, Position};
use crate::entity::item::{ItemBaseInfo, ItemBattleStats};
use crate::entity::{CommonEntity, GetEditParams};
use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use strum_macros::{Display, EnumIter};

impl GetEditParams<()> for Weapon {
    fn edit_params(&self) {}
}

impl CommonEntity<ItemId> for Weapon {
    fn name(&self) -> String {
        self.base_info.name.clone()
    }

    fn desc(&self) -> String {
        self.base_info.desc.clone()
    }

    fn id(&self) -> ItemId {
        self.base_info.id
    }

    fn changed(&self) -> bool {
        self._changed
    }

    fn deleted(&self) -> bool {
        self._deleted
    }

    fn new(id: ItemId) -> Self {
        let mut s = Self::default();

        s.base_info.id = id;
        s.base_info.name = "New Weapon".to_string();

        s
    }
}

#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct Weapon {
    pub(crate) base_info: ItemBaseInfo,

    pub(crate) weapon_type: WeaponType,
    pub(crate) character_animation_type: CharacterAnimationType,
    pub(crate) mp_consume: WeaponMpConsume,
    pub(crate) random_damage: RandomDamage,
    pub(crate) ertheia_fists_scale: f32,

    pub(crate) mesh_info: Vec<WeaponMeshInfo>,
    pub(crate) sound: Vec<String>,
    pub(crate) effect: String,

    pub(crate) soulshot_count: u8,
    pub(crate) spiritshot_count: u8,
    pub(crate) curvature: i16,

    pub(crate) unk: bool,
    pub(crate) is_hero_weapon: bool,
    pub(crate) is_magic_weapon: bool,
    pub(crate) can_ensoul: bool,
    pub(crate) ensoul_count: u8,

    pub(crate) enchant_info: WindowParams<WeaponEnchantInfo, (), WeaponEnchantAction, ()>,

    pub(crate) variation_info: WindowParams<WeaponVariationInfo, (), WeaponVariationAction, ()>,

    pub _changed: bool,
    pub _deleted: bool,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct WeaponEnchantInfo {
    pub(crate) junk: i16,
    pub(crate) params: Vec<WeaponEnchantParams>,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct WeaponVariationInfo {
    pub(crate) icon: Vec<String>,
    pub(crate) effect_1: u8,
    pub(crate) effect_2: u8,
    pub(crate) effect_3: u8,
    pub(crate) effect_4: u8,
    pub(crate) effect_5: u8,
    pub(crate) effect_6: u8,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct WeaponMeshInfo {
    pub(crate) mesh: String,
    pub(crate) unk: u8,
    pub(crate) texture: String,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct WeaponEnchantParams {
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
    Unk10 = 10,
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
