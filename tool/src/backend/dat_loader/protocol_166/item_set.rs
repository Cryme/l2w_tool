use crate::backend::log_holder::Log;
use crate::common::ItemId;
use crate::entity::item_set::{ItemSet, ItemSetEnchantInfo};
use std::collections::HashMap;

use l2_rw::ue2_rw::{ASCF, DWORD, UVEC};
use l2_rw::{DatVariant, deserialize_dat, save_dat};

use l2_rw::ue2_rw::{ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal};

use crate::backend::Localization;
use crate::backend::dat_loader::{GetId, wrap_into_id_map};
use crate::backend::holder::{GameDataHolder, HolderMapOps};
use r#macro::{ReadUnreal, WriteUnreal};
use std::thread;
use std::thread::JoinHandle;

impl ItemSetGrpDat {
    fn from_entity(set: &ItemSet, localization: Localization) -> Self {
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
                .map(|v| (&v[localization]).into())
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
                .map(|v| (&v[localization]).into())
                .collect::<Vec<ASCF>>()
                .into(),
            unk1: set.unk1,
            unk2: set.unk2,
            enchant_bonuses: set
                .enchant_info
                .iter()
                .map(|v| DatEnchantBonus {
                    enchant_level: v.enchant_level,
                    description: (&v.enchant_description[localization]).into(),
                })
                .collect::<Vec<DatEnchantBonus>>()
                .into(),
        }
    }
}

impl GameDataHolder {
    pub fn serialize_item_sets_to_binary(&mut self) -> JoinHandle<Vec<Log>> {
        let set_grp_ru: Vec<ItemSetGrpDat> = self
            .item_set_holder
            .values()
            .filter(|v| !v._deleted)
            .map(|v| ItemSetGrpDat::from_entity(v, Localization::RU))
            .collect();

        let set_grp_path_ru = self
            .dat_paths
            .get(&"setitemgrp-ru.dat".to_string())
            .unwrap()
            .clone();

        let eu = if let Some(set_grp_path_eu) = self.dat_paths.get(&"setitemgrp-eu.dat".to_string())
        {
            let set_grp_eu: Vec<ItemSetGrpDat> = self
                .item_set_holder
                .values()
                .filter(|v| !v._deleted)
                .map(|v| ItemSetGrpDat::from_entity(v, Localization::EU))
                .collect();

            Some((set_grp_eu, set_grp_path_eu.clone()))
        } else {
            None
        };

        thread::spawn(move || {
            let mut log = vec![if let Err(e) = save_dat(
                set_grp_path_ru.path(),
                DatVariant::<(), ItemSetGrpDat>::Array(set_grp_ru),
            ) {
                Log::from_loader_e(e)
            } else {
                Log::from_loader_i("Set Item Grp RU saved")
            }];

            if let Some((set_grp_eu, set_grp_path_eu)) = eu {
                log.push(
                    if let Err(e) = save_dat(
                        set_grp_path_eu.path(),
                        DatVariant::<(), ItemSetGrpDat>::Array(set_grp_eu),
                    ) {
                        Log::from_loader_e(e)
                    } else {
                        Log::from_loader_i("Set Item Grp EU saved")
                    },
                )
            }

            log
        })
    }

    pub fn load_item_sets(&mut self) -> Result<Vec<Log>, ()> {
        let set_grp_ru = deserialize_dat::<ItemSetGrpDat>(
            self.dat_paths
                .get(&"setitemgrp-ru.dat".to_string())
                .unwrap()
                .path(),
        )?;

        let set_grp_eu = if let Some(eu_path) = self.dat_paths.get(&"setitemgrp-eu.dat".to_string())
        {
            wrap_into_id_map(deserialize_dat::<ItemSetGrpDat>(eu_path.path())?)
        } else {
            HashMap::new()
        };

        for v in set_grp_ru {
            let eu = set_grp_eu.get(&v.id);

            self.item_set_holder.insert(
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
                        .enumerate()
                        .map(|(i, v)| {
                            (
                                v.to_string(),
                                eu.map_or("NOT_EXITS".to_string(), |vv| {
                                    vv.base_descriptions
                                        .inner
                                        .get(i)
                                        .map_or("NOT_EXITS".to_string(), |v| v.to_string())
                                }),
                            )
                                .into()
                        })
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
                        .enumerate()
                        .map(|(i, v)| {
                            (
                                v.to_string(),
                                eu.map_or("NOT_EXITS".to_string(), |vv| {
                                    vv.additional_descriptions
                                        .inner
                                        .get(i)
                                        .map_or("NOT_EXITS".to_string(), |v| v.to_string())
                                }),
                            )
                                .into()
                        })
                        .collect(),
                    unk1: v.unk1,
                    unk2: v.unk2,
                    enchant_info: v
                        .enchant_bonuses
                        .inner
                        .iter()
                        .enumerate()
                        .map(|(i, v)| ItemSetEnchantInfo {
                            enchant_level: v.enchant_level,
                            enchant_description: (
                                v.description.to_string(),
                                eu.map_or("NOT_EXITS".to_string(), |vv| {
                                    vv.enchant_bonuses
                                        .inner
                                        .get(i)
                                        .map_or("NOT_EXITS".to_string(), |v| {
                                            v.description.to_string()
                                        })
                                }),
                            )
                                .into(),
                        })
                        .collect(),
                    ..Default::default()
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
