use crate::backend::editor::WindowParams;
use crate::backend::entity_impl::item::weapon::{WeaponEnchantAction, WeaponVariationAction};
use crate::common::{ItemId, Position, SetEnchantEffectId};
use crate::entity::item::{ItemBaseInfo, ItemBattleStats};
use crate::entity::{CommonEntity, GetEditParams};
use num_derive::{FromPrimitive, ToPrimitive};
use rhai::{CustomType, TypeBuilder};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::ops::{Index, IndexMut};
use strum_macros::{Display, EnumIter};

impl GetEditParams<()> for Armor {
    fn edit_params(&self) {}
}

impl CommonEntity<ItemId> for Armor {
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
        s.base_info.name = "New EtcItem".to_string();

        s
    }
}

#[derive(Serialize, Deserialize, Display, Debug, EnumIter, Eq, PartialEq, Copy, Clone, Default)]
#[repr(u8)]
pub enum CurrentArmorMesh {
    #[default]
    MHumanFighter,
    FHumanFighter,
    MDarkElf,
    FDarkElf,
    MDwarf,
    FDwarf,
    MElf,
    FElf,
    MHumanMystic,
    FHumanMystic,
    MOrcFighter,
    FOrcFighter,
    MOrcMystic,
    FOrcMystic,
    MKamael,
    FKamael,
    MErtheia,
    FErtheia,
    Npc,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq, CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct Armor {
    pub(crate) base_info: ItemBaseInfo,

    pub(crate) armor_type: ArmorType,
    pub(crate) attack_effect: String,
    pub(crate) unk1: u32,
    pub(crate) unk2: bool,
    pub(crate) mp_bonus: u16,
    pub(crate) hide_mask: u16,
    pub(crate) item_sound: Vec<String>,

    pub(crate) underwater_body_type1: UnderwaterBodyType1,
    pub(crate) underwater_body_type2: UnderwaterBodyType2,
    pub(crate) set_enchant_effect_id: SetEnchantEffectId,

    pub(crate) mesh_info: WindowParams<ArmorMeshes, (), (), CurrentArmorMesh>,

    #[serde(skip)]
    pub _changed: bool,
    #[serde(skip)]
    pub _deleted: bool,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq, CustomType)]
pub struct ArmorMeshes {
    pub(crate) m_human_fighter: ArmorMeshInfo,
    pub(crate) f_human_fighter: ArmorMeshInfo,
    pub(crate) m_dark_elf: ArmorMeshInfo,
    pub(crate) f_dark_elf: ArmorMeshInfo,
    pub(crate) m_dwarf: ArmorMeshInfo,
    pub(crate) f_dwarf: ArmorMeshInfo,
    pub(crate) m_elf: ArmorMeshInfo,
    pub(crate) f_elf: ArmorMeshInfo,
    pub(crate) m_human_mystic: ArmorMeshInfo,
    pub(crate) f_human_mystic: ArmorMeshInfo,
    pub(crate) m_orc_fighter: ArmorMeshInfo,
    pub(crate) f_orc_fighter: ArmorMeshInfo,
    pub(crate) m_orc_mystic: ArmorMeshInfo,
    pub(crate) f_orc_mystic: ArmorMeshInfo,
    pub(crate) m_kamael: ArmorMeshInfo,
    pub(crate) f_kamael: ArmorMeshInfo,
    pub(crate) m_ertheia: ArmorMeshInfo,
    pub(crate) f_ertheia: ArmorMeshInfo,
    pub(crate) npc: ArmorMeshInfo,
}

impl Index<CurrentArmorMesh> for ArmorMeshes {
    type Output = ArmorMeshInfo;

    fn index(&self, index: CurrentArmorMesh) -> &Self::Output {
        match index {
            CurrentArmorMesh::MHumanFighter => &self.m_human_fighter,
            CurrentArmorMesh::FHumanFighter => &self.f_human_fighter,
            CurrentArmorMesh::MDarkElf => &self.m_dark_elf,
            CurrentArmorMesh::FDarkElf => &self.f_dark_elf,
            CurrentArmorMesh::MDwarf => &self.m_dwarf,
            CurrentArmorMesh::FDwarf => &self.f_dwarf,
            CurrentArmorMesh::MElf => &self.m_elf,
            CurrentArmorMesh::FElf => &self.f_elf,
            CurrentArmorMesh::MHumanMystic => &self.m_human_mystic,
            CurrentArmorMesh::FHumanMystic => &self.f_human_mystic,
            CurrentArmorMesh::MOrcFighter => &self.m_orc_fighter,
            CurrentArmorMesh::FOrcFighter => &self.f_orc_fighter,
            CurrentArmorMesh::MOrcMystic => &self.m_orc_mystic,
            CurrentArmorMesh::FOrcMystic => &self.f_orc_mystic,
            CurrentArmorMesh::MKamael => &self.m_kamael,
            CurrentArmorMesh::FKamael => &self.f_kamael,
            CurrentArmorMesh::MErtheia => &self.m_ertheia,
            CurrentArmorMesh::FErtheia => &self.f_ertheia,
            CurrentArmorMesh::Npc => &self.npc,
        }
    }
}

impl IndexMut<CurrentArmorMesh> for ArmorMeshes {
    fn index_mut(&mut self, index: CurrentArmorMesh) -> &mut Self::Output {
        match index {
            CurrentArmorMesh::MHumanFighter => &mut self.m_human_fighter,
            CurrentArmorMesh::FHumanFighter => &mut self.f_human_fighter,
            CurrentArmorMesh::MDarkElf => &mut self.m_dark_elf,
            CurrentArmorMesh::FDarkElf => &mut self.f_dark_elf,
            CurrentArmorMesh::MDwarf => &mut self.m_dwarf,
            CurrentArmorMesh::FDwarf => &mut self.f_dwarf,
            CurrentArmorMesh::MElf => &mut self.m_elf,
            CurrentArmorMesh::FElf => &mut self.f_elf,
            CurrentArmorMesh::MHumanMystic => &mut self.m_human_mystic,
            CurrentArmorMesh::FHumanMystic => &mut self.f_human_mystic,
            CurrentArmorMesh::MOrcFighter => &mut self.m_orc_fighter,
            CurrentArmorMesh::FOrcFighter => &mut self.f_orc_fighter,
            CurrentArmorMesh::MOrcMystic => &mut self.m_orc_mystic,
            CurrentArmorMesh::FOrcMystic => &mut self.f_orc_mystic,
            CurrentArmorMesh::MKamael => &mut self.m_kamael,
            CurrentArmorMesh::FKamael => &mut self.f_kamael,
            CurrentArmorMesh::MErtheia => &mut self.m_ertheia,
            CurrentArmorMesh::FErtheia => &mut self.f_ertheia,
            CurrentArmorMesh::Npc => &mut self.npc,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq, CustomType)]
pub struct ArmorMeshBase {
    pub(crate) unk1: Vec<String>,
    pub(crate) unk2: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq, CustomType)]
pub struct ArmorMeshAdditionalF {
    pub(crate) unk2: String,
    pub(crate) unk3: u8,
    pub(crate) unk4: u8,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq, CustomType)]
pub struct ArmorMeshAdditional {
    pub(crate) unk1: Vec<ArmorMeshAdditionalF>,
    pub(crate) unk5: Vec<String>,
    pub(crate) unk6: String,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq, CustomType)]
pub struct ArmorMeshInfo {
    pub(crate) base: ArmorMeshBase,
    pub(crate) additional: ArmorMeshAdditional,
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
pub enum ArmorType {
    #[default]
    Unk0 = 0,
    Unk1 = 1,
    Unk2 = 2,
    Unk3 = 3,
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
pub enum UnderwaterBodyType1 {
    #[default]
    Unk0 = 0,
    Unk1 = 1,
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
pub enum UnderwaterBodyType2 {
    #[default]
    Unk0 = 0,
    Unk1 = 1,
    Unk2 = 2,
    Unk3 = 3,
    Unk4 = 4,
    Unk5 = 5,
}
