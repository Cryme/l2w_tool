mod weapon;

use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};
use crate::data::ItemId;

#[derive(Clone)]
pub struct Item {
    pub(crate) id: ItemId,
    pub(crate) name: String,
    pub(crate) desc: String,
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
    Unk4 = 4,
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
pub enum MaterialType {
    #[default]
    Unk1 = 1,
    Unk2 = 2,
    Unk4 = 4,
    Unk6 = 6,
    Unk8 = 8,
    Unk13 = 13,
    Unk14 = 14,
    Unk17 = 17,
    Unk18 = 18,
    Unk19 = 19,
    Unk23 = 23,
    Unk47 = 47,
    Unk48 = 48,
    Unk49 = 49,
    Unk50 = 50,
    Unk51 = 51,
    Unk52 = 52,
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
    TwoHandedWeapon = 7,
    OneHandedWeapon = 34,
    Shield = 35,
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
pub enum DropAnimationType {
    #[default]
    Unk0,
    Unk1,
    Unk2,
    Unk3,
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
pub enum QualityType {
    #[default]
    Common,
    Normal,
    Rare,
    Epic,
    Blessed,
    Dragon,
}
