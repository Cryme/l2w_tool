use crate::backend::item::weapon::{WeaponEnchantAction, WeaponVariationAction};
use crate::backend::WindowParams;
use crate::data::{ItemId, Position};
use crate::entity::item::{ItemBaseInfo, ItemBattleStats};
use crate::entity::CommonEntity;
use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use strum_macros::{Display, EnumIter};

impl CommonEntity<ItemId, ()> for EtcItem {
    fn name(&self) -> String {
        self.base_info.name.clone()
    }

    fn desc(&self) -> String {
        self.base_info.desc.clone()
    }

    fn id(&self) -> ItemId {
        self.base_info.id
    }

    fn edit_params(&self) {}

    fn new(id: ItemId) -> Self {
        let mut s = Self::default();

        s.base_info.id = id;
        s.base_info.name = "New EtcItem".to_string();

        s
    }
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct EtcItem {
    pub(crate) base_info: ItemBaseInfo,
    pub(crate) battle_stats: WindowParams<ItemBattleStats, (), (), ()>,
    pub(crate) etc_item_type: EtcItemType,
    pub(crate) consume_type: ConsumeType,

    pub(crate) mesh_info: Vec<EtcMeshInfo>,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct EtcMeshInfo {
    pub(crate) mesh: String,
    pub(crate) texture: String,
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
pub enum EtcItemType {
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
    Unk59 = 59,
    Unk60 = 60,
    Unk61 = 61,
    Unk62 = 62,
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
pub enum ConsumeType {
    #[default]
    Unk0 = 0,
    Unk1 = 1,
    Unk2 = 2,
    Unk3 = 3,
    Unk5 = 5,
    Unk6 = 6,
    Unk7 = 7,
    Unk8 = 8,
    Unk9 = 9,
    Unk10 = 10,
    Unk11 = 11,
}
