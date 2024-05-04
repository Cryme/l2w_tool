use crate::backend::Log;
use crate::dat_loader::grand_crusade_110::{L2GeneralStringTable, Loader110};
use crate::data::ItemId;
use crate::entity::item_set::{ItemSet, ItemSetEnchantInfo};

use l2_rw::ue2_rw::{ASCF, DWORD, UVEC};
use l2_rw::{deserialize_dat, save_dat, DatVariant};

use l2_rw::ue2_rw::{ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal};

use crate::dat_loader::GetId;
use r#macro::{ReadUnreal, WriteUnreal};
use std::thread;
use std::thread::JoinHandle;

impl From<(&ItemSet, &mut L2GeneralStringTable)> for ItemSetGrpDat {
    fn from(value: (&ItemSet, &mut L2GeneralStringTable)) -> Self {
        let (set, _table) = value;

        ItemSetGrpDat {
            id: set.id.0,
            base_item_ids: set
                .base_items
                .iter()
                .map(|v| v.iter().map(|vv| vv.0).collect::<Vec<u32>>().into())
                .collect::<Vec<UVEC<DWORD, DWORD>>>()
                .into(),
            base_descriptions: set
                .base_descriptions
                .iter()
                .map(|v| v.into())
                .collect::<Vec<ASCF>>()
                .into(),
            additional_item_ids: set
                .additional_items
                .iter()
                .map(|v| v.iter().map(|vv| vv.0).collect::<Vec<u32>>().into())
                .collect::<Vec<UVEC<DWORD, DWORD>>>()
                .into(),
            additional_descriptions: set
                .additional_descriptions
                .iter()
                .map(|v| v.into())
                .collect::<Vec<ASCF>>()
                .into(),
            unk1: set.unk1,
            unk2: set.unk2,
            enchant_bonuses: set
                .enchant_info
                .iter()
                .map(|v| DatEnchantBonus {
                    enchant_level: v.enchant_level,
                    description: (&v.enchant_description).into(),
                })
                .collect::<Vec<DatEnchantBonus>>()
                .into(),
        }
    }
}

impl Loader110 {
    pub fn serialize_item_sets_to_binary(&mut self) -> JoinHandle<Log> {
        let mut set_grp: Vec<ItemSetGrpDat> = vec![];

        for set in self.item_sets.values() {
            set_grp.push((set, &mut self.game_data_name).into());
        }

        let set_grp_path = self
            .dat_paths
            .get(&"setitemgrp-ru.dat".to_string())
            .unwrap()
            .clone();

        thread::spawn(move || {
            if let Err(e) =save_dat(
                set_grp_path.path(),
                DatVariant::<(), ItemSetGrpDat>::Array(set_grp.to_vec()),
            ) {
                Log::from_loader_e(e)
            } else {
                Log::from_loader_i("Set Item Grp saved")
            }
        })
    }

    pub fn load_item_sets(&mut self) -> Result<Vec<Log>, ()> {
        let set_grp = deserialize_dat::<ItemSetGrpDat>(
            self.dat_paths
                .get(&"setitemgrp-ru.dat".to_string())
                .unwrap()
                .path(),
        )?;

        for v in set_grp {
            self.item_sets.insert(
                v.id.into(),
                ItemSet {
                    id: v.id.into(),
                    base_items: v
                        .base_item_ids
                        .inner
                        .iter()
                        .map(|v| v.inner.iter().map(|vv| ItemId(*vv)).collect())
                        .collect(),
                    base_descriptions: v
                        .base_descriptions
                        .inner
                        .iter()
                        .map(|v| v.to_string())
                        .collect(),
                    additional_items: v
                        .additional_item_ids
                        .inner
                        .iter()
                        .map(|v| v.inner.iter().map(|vv| ItemId(*vv)).collect())
                        .collect(),
                    additional_descriptions: v
                        .additional_descriptions
                        .inner
                        .iter()
                        .map(|v| v.to_string())
                        .collect(),
                    unk1: v.unk1,
                    unk2: v.unk2,
                    enchant_info: v
                        .enchant_bonuses
                        .inner
                        .iter()
                        .map(|v| ItemSetEnchantInfo {
                            enchant_level: v.enchant_level,
                            enchant_description: v.description.to_string(),
                        })
                        .collect(),
                },
            );
        }

        Ok(vec![])
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct DatEnchantBonus {
    enchant_level: DWORD,
    description: ASCF,
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct ItemSetGrpDat {
    id: DWORD,
    base_item_ids: UVEC<DWORD, UVEC<DWORD, DWORD>>,
    base_descriptions: UVEC<DWORD, ASCF>,
    additional_item_ids: UVEC<DWORD, UVEC<DWORD, DWORD>>,
    additional_descriptions: UVEC<DWORD, ASCF>,
    unk1: DWORD,
    unk2: DWORD,
    enchant_bonuses: UVEC<DWORD, DatEnchantBonus>,
}

impl GetId for ItemSetGrpDat {
    #[inline(always)]
    fn get_id(&self) -> u32 {
        self.id
    }
}
