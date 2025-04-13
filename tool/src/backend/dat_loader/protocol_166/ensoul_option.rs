use crate::backend::log_holder::Log;
use std::collections::{HashMap, HashSet};

use l2_rw::ue2_rw::{ASCF, DWORD};
use l2_rw::ue2_rw::{ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal};
use l2_rw::{DatVariant, deserialize_dat, save_dat};

use crate::backend::Localization;
use crate::backend::dat_loader::{GetId, wrap_into_id_map};
use crate::backend::holder::{GameDataHolder, HolderMapOps};
use crate::common::ItemId;
use crate::entity::ensoul_option::EnsoulOption;
use r#macro::{ReadUnreal, WriteUnreal};
use std::thread::JoinHandle;
use std::{thread, vec};

impl GameDataHolder {
    fn as_dat_vec(&mut self, localization: Localization) -> Vec<EnsoulOptionClientDat> {
        self.ensoul_option_holder
            .values()
            .filter(|v| !v._deleted)
            .map(|v| EnsoulOptionClientDat {
                option_type: v.option_type,
                step: v.step,
                id: v.id.0,
                name: (&v.name[localization]).into(),
                desc: (&v.desc[localization]).into(),
                extraction_item_id: v.extraction_item_id.0,
                icon: self.game_string_table_ru.get_index(&v.icon),
                icon_panel: self.game_string_table_ru.get_index(&v.icon_panel),
            })
            .collect()
    }

    pub fn serialize_ensoul_option_to_binary(&mut self) -> JoinHandle<Vec<Log>> {
        let ensoul_options_ru = self.as_dat_vec(Localization::RU);

        let dat_path_ru = self
            .dat_paths
            .get(&"ensoul_option_client-eu.dat".to_string())
            .unwrap()
            .clone();

        let eu = if let Some(dat_path_eu) = self
            .dat_paths
            .get(&"ensoul_option_client-eu.dat".to_string())
        {
            let path_eu = dat_path_eu.clone();
            let ensoul_options_eu = self.as_dat_vec(Localization::EU);

            Some((path_eu, ensoul_options_eu))
        } else {
            None
        };

        thread::spawn(move || {
            let mut log = if let Err(e) = save_dat(
                dat_path_ru.path(),
                DatVariant::<(), EnsoulOptionClientDat>::Array(ensoul_options_ru.to_vec()),
            ) {
                vec![Log::from_loader_e(e)]
            } else {
                vec![Log::from_loader_i("Ensoul Options saved")]
            };

            if let Some((path_eu, ensoul_options_eu)) = eu {
                log.push(
                    if let Err(e) = save_dat(
                        path_eu.path(),
                        DatVariant::<(), EnsoulOptionClientDat>::Array(ensoul_options_eu.to_vec()),
                    ) {
                        Log::from_loader_e(e)
                    } else {
                        Log::from_loader_i("Ensoul Options saved")
                    },
                );
            };

            log
        })
    }

    pub fn load_ensoul_options(&mut self) -> Result<Vec<Log>, ()> {
        let options_ru = deserialize_dat::<EnsoulOptionClientDat>(
            self.dat_paths
                .get(&"ensoul_option_client-ru.dat".to_string())
                .unwrap()
                .path(),
        )?;

        let options_eu = if let Some(eu_path) = self
            .dat_paths
            .get(&"ensoul_option_client-eu.dat".to_string())
        {
            wrap_into_id_map(deserialize_dat::<EnsoulOptionClientDat>(eu_path.path())?)
        } else {
            HashMap::new()
        };

        let mut step = HashSet::new();
        let mut o_type = HashSet::new();

        for v in options_ru {
            o_type.insert(v.option_type);
            step.insert(v.step);

            self.ensoul_option_holder.insert(
                v.id.into(),
                EnsoulOption {
                    id: v.id.into(),
                    option_type: v.option_type,
                    step: v.step,
                    name: (
                        v.name.to_string(),
                        options_eu
                            .get(&v.id)
                            .map_or("NOT_EXIST".to_string(), |v| v.name.to_string()),
                    )
                        .into(),
                    desc: (
                        v.desc.to_string(),
                        options_eu
                            .get(&v.id)
                            .map_or("NOT_EXIST".to_string(), |v| v.desc.to_string()),
                    )
                        .into(),
                    extraction_item_id: ItemId(v.extraction_item_id),
                    icon: self.game_string_table_ru.get_o(&v.icon),
                    icon_panel: self.game_string_table_ru.get_o(&v.icon_panel),

                    _changed: false,
                    _deleted: false,
                },
            );
        }

        Ok(vec![])
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct EnsoulOptionClientDat {
    option_type: DWORD,
    step: DWORD,
    id: DWORD,
    name: ASCF,
    desc: ASCF,
    extraction_item_id: DWORD,
    icon: DWORD,
    icon_panel: DWORD,
}

impl GetId for EnsoulOptionClientDat {
    fn get_id(&self) -> DWORD {
        self.id
    }
}
