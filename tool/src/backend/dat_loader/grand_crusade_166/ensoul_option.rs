use crate::backend::log_holder::Log;
use std::collections::HashSet;

use l2_rw::ue2_rw::{ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal};
use l2_rw::ue2_rw::{ASCF, DWORD};
use l2_rw::{deserialize_dat, save_dat, DatVariant};

use crate::backend::dat_loader::L2StringTable;
use crate::backend::holder::{GameDataHolder, HolderMapOps};
use crate::common::ItemId;
use crate::entity::ensoul_option::EnsoulOption;
use r#macro::{ReadUnreal, WriteUnreal};
use std::thread;
use std::thread::JoinHandle;

impl GameDataHolder {
    pub fn serialize_ensoul_option_to_binary(&mut self) -> JoinHandle<Log> {
        let ensoul_options: Vec<EnsoulOptionClientDat> = self
            .ensoul_option_holder
            .values()
            .filter(|v| !v._deleted)
            .map(|v| EnsoulOptionClientDat {
                option_type: v.option_type,
                step: v.step,
                id: v.id.0,
                name: (&v.name).into(),
                desc: (&v.desc).into(),
                extraction_item_id: v.extraction_item_id.0,
                icon: self.game_string_table.get_index(&v.icon),
                icon_panel: self.game_string_table.get_index(&v.icon_panel),
            })
            .collect();

        let dat_path = self
            .dat_paths
            .get(&"ensoul_option_client-ru.dat".to_string())
            .unwrap()
            .clone();

        thread::spawn(move || {
            if let Err(e) = save_dat(
                dat_path.path(),
                DatVariant::<(), EnsoulOptionClientDat>::Array(ensoul_options.to_vec()),
            ) {
                Log::from_loader_e(e)
            } else {
                Log::from_loader_i("Ensoul Options saved")
            }
        })
    }

    pub fn load_ensoul_options(&mut self) -> Result<Vec<Log>, ()> {
        let options = deserialize_dat::<EnsoulOptionClientDat>(
            self.dat_paths
                .get(&"ensoul_option_client-ru.dat".to_string())
                .unwrap()
                .path(),
        )?;

        let mut step = HashSet::new();
        let mut o_type = HashSet::new();

        for v in options {
            o_type.insert(v.option_type);
            step.insert(v.step);

            self.ensoul_option_holder.insert(
                v.id.into(),
                EnsoulOption {
                    id: v.id.into(),
                    option_type: v.option_type,
                    step: v.step,
                    name: v.name.to_string(),
                    desc: v.desc.to_string(),
                    extraction_item_id: ItemId(v.extraction_item_id),
                    icon: self.game_string_table.get_o(&v.icon),
                    icon_panel: self.game_string_table.get_o(&v.icon_panel),

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
