use crate::backend::util::StringCow;
use crate::common::{EnsoulOptionId, ItemId};
use crate::entity::item::ItemBaseInfo;
use crate::entity::{CommonEntity, GetEditParams};
use num_derive::{FromPrimitive, ToPrimitive};
use rhai::{CustomType, TypeBuilder};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use strum_macros::{Display, EnumIter};

impl GetEditParams<()> for EtcItem {
    fn edit_params(&self) {}
}

impl CommonEntity<ItemId> for EtcItem {
    fn name(&self) -> String {
        self.base_info.name.ru.to_string()
    }

    fn desc(&self) -> String {
        self.base_info.desc.ru.clone()
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
        s.base_info.name.ru = "Новый EtcItem".into();
        s.base_info.name.eu = "New EtcItem".into();

        s
    }
}

#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq, CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct EtcItem {
    pub(crate) base_info: ItemBaseInfo,
    pub(crate) etc_item_type: EtcItemType,
    pub(crate) consume_type: ConsumeType,
    pub(crate) ensoul_stone: Option<EnsoulStone>,

    pub(crate) mesh_info: Vec<EtcMeshInfo>,

    #[serde(skip)]
    pub _changed: bool,
    #[serde(skip)]
    pub _deleted: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialEq, CustomType)]
pub struct EnsoulStone {
    pub slot_type: EnsoulSlotType,
    pub options: Vec<EnsoulOptionId>,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq, CustomType)]
pub struct EtcMeshInfo {
    pub(crate) mesh: StringCow,
    pub(crate) texture: StringCow,
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
pub enum EnsoulSlotType {
    #[default]
    Unk1 = 1,
    Unk2 = 2,
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
    Unk63 = 63,
    Unk64 = 64,
    Unk65 = 65,
    Unk66 = 66,
    Unk67 = 67,
    Unk68 = 68,
    Unk69 = 69,
    Unk70 = 70,
    Unk71 = 71,
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
