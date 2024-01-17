use std::fmt::{Display, Formatter};
use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};
use crate::entity::item::ItemBaseInfo;

#[derive(Clone)]
pub struct Weapon {
    base_info: ItemBaseInfo,
    weapon_type: WeaponType,
    hand_type: HandType,
    character_animation_type: CharacterAnimationType,
    random_damage: RandomDamage,

    p_defense: u16,
    m_defense: u16,
    p_attack: u16,
    m_attack: u16,
    p_attack_speed: u16,
    p_hit: f32,
    m_hit: f32,
    p_critical: f32,
    m_critical: f32,
    speed: u8,
    shield_defense: u16,
    shield_defense_rate: u8,
    p_avoid: f32,
    m_avoid: f32,
    property_params: u16,


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
