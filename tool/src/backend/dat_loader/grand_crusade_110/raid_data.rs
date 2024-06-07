use crate::backend::dat_loader::grand_crusade_110::Loader110;
use crate::backend::log_holder::Log;

use l2_rw::ue2_rw::{ASCF, BYTE, DWORD, FLOAT};
use l2_rw::{deserialize_dat, save_dat, DatVariant};

use l2_rw::ue2_rw::{ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal};

use crate::backend::dat_loader::GetId;
use crate::backend::holder::HolderMapOps;
use crate::data::Position;
use crate::entity::raid_info::RaidInfo;
use r#macro::{ReadUnreal, WriteUnreal};
use std::thread;
use std::thread::JoinHandle;

impl Loader110 {
    pub fn serialize_raid_data_to_binary(&mut self) -> JoinHandle<Log> {
        let raid_grp: Vec<RaidDataDat> = self
            .raid_info
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
                desc: ASCF::from(&v.desc),
                recommended_level_min: v.recommended_level_min,
                recommended_level_max: v.recommended_level_max,
            })
            .collect();

        let dat_path = self
            .dat_paths
            .get(&"raiddata-ru.dat".to_string())
            .unwrap()
            .clone();

        thread::spawn(move || {
            if let Err(e) = save_dat(
                dat_path.path(),
                DatVariant::<(), RaidDataDat>::Array(raid_grp.to_vec()),
            ) {
                Log::from_loader_e(e)
            } else {
                Log::from_loader_i("RaidData saved")
            }
        })
    }

    pub fn load_raid_data(&mut self) -> Result<Vec<Log>, ()> {
        let raid_grp = deserialize_dat::<RaidDataDat>(
            self.dat_paths
                .get(&"raiddata-ru.dat".to_string())
                .unwrap()
                .path(),
        )?;

        for v in raid_grp {
            self.raid_info.insert(
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
                    desc: v.desc.to_string(),
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
