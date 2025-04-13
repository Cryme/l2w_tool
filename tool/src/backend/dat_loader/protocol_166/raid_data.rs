use crate::backend::log_holder::Log;
use std::collections::HashMap;

use l2_rw::ue2_rw::{ASCF, BYTE, DWORD, FLOAT};
use l2_rw::{DatVariant, deserialize_dat, save_dat};

use l2_rw::ue2_rw::{ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal};

use crate::backend::Localization;
use crate::backend::dat_loader::{GetId, wrap_into_id_map};
use crate::backend::holder::{GameDataHolder, HolderMapOps};
use crate::common::Position;
use crate::entity::raid_info::RaidInfo;
use r#macro::{ReadUnreal, WriteUnreal};
use std::thread;
use std::thread::JoinHandle;

impl GameDataHolder {
    fn as_dat(&mut self, localization: Localization) -> Vec<RaidDataDat> {
        self.raid_info_holder
            .values()
            .filter(|v| !v._deleted)
            .map(|v| RaidDataDat {
                id: v.id.0,
                raid_id: v.id.0,
                raid_lvl: v.raid_lvl,
                search_zone_id: v.search_zone_id.0,
                x: v.loc.x,
                y: v.loc.y,
                z: v.loc.z,
                desc: ASCF::from(&v.desc[localization]),
                recommended_level_min: v.recommended_level_min,
                recommended_level_max: v.recommended_level_max,
            })
            .collect()
    }

    pub fn serialize_raid_data_to_binary(&mut self) -> JoinHandle<Vec<Log>> {
        let raid_grp = self.as_dat(Localization::RU);

        let dat_path_ru = self
            .dat_paths
            .get(&"raiddata-ru.dat".to_string())
            .unwrap()
            .clone();

        let eu = if let Some(dir) = self.dat_paths.get(&"raiddata-eu.dat".to_string()).cloned() {
            Some((self.as_dat(Localization::EU), dir))
        } else {
            None
        };

        thread::spawn(move || {
            let mut log = vec![if let Err(e) = save_dat(
                dat_path_ru.path(),
                DatVariant::<(), RaidDataDat>::Array(raid_grp),
            ) {
                Log::from_loader_e(e)
            } else {
                Log::from_loader_i("RaidData RU saved")
            }];

            if let Some((dats, dir)) = eu {
                log.push(
                    if let Err(e) = save_dat(dir.path(), DatVariant::<(), RaidDataDat>::Array(dats))
                    {
                        Log::from_loader_e(e)
                    } else {
                        Log::from_loader_i("RaidData EU saved")
                    },
                )
            }

            log
        })
    }

    pub fn load_raid_data(&mut self) -> Result<Vec<Log>, ()> {
        let raid_grp_ru = deserialize_dat::<RaidDataDat>(
            self.dat_paths
                .get(&"raiddata-ru.dat".to_string())
                .unwrap()
                .path(),
        )?;

        let raid_grp_eu = if let Some(dir) = self.dat_paths.get(&"raiddata-eu.dat".to_string()) {
            wrap_into_id_map(deserialize_dat::<RaidDataDat>(dir.path())?)
        } else {
            HashMap::new()
        };

        for v in raid_grp_ru {
            self.raid_info_holder.insert(
                v.id.into(),
                RaidInfo {
                    id: v.id.into(),
                    raid_id: v.raid_id.into(),
                    raid_lvl: v.raid_lvl,
                    search_zone_id: v.search_zone_id.into(),
                    loc: Position {
                        x: v.x,
                        y: v.y,
                        z: v.z,
                    },
                    desc: (
                        v.desc.to_string(),
                        raid_grp_eu
                            .get(&v.id)
                            .map_or("NOT_EXIST".to_string(), |v| v.desc.to_string()),
                    )
                        .into(),
                    recommended_level_min: v.recommended_level_min,
                    recommended_level_max: v.recommended_level_max,

                    _changed: false,
                    _deleted: false,
                },
            );
        }

        Ok(vec![])
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct RaidDataDat {
    id: DWORD,
    raid_id: DWORD,
    raid_lvl: DWORD,
    search_zone_id: DWORD,
    x: FLOAT,
    y: FLOAT,
    z: FLOAT,
    desc: ASCF,
    recommended_level_min: BYTE,
    recommended_level_max: BYTE,
}

impl GetId for RaidDataDat {
    #[inline(always)]
    fn get_id(&self) -> u32 {
        self.id
    }
}
