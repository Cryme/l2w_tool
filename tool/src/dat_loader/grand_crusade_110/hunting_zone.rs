use crate::backend::Log;
use crate::dat_loader::grand_crusade_110::{CoordsXYZ, L2GeneralStringTable, Loader110};

use l2_rw::ue2_rw::{ASCF, DWORD, USHORT};
use l2_rw::{deserialize_dat, save_dat, DatVariant};

use l2_rw::ue2_rw::{ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal};

use crate::data::QuestId;
use crate::entity::hunting_zone::{HuntingZone, HuntingZoneType};
use num_traits::{FromPrimitive, ToPrimitive};
use r#macro::{ReadUnreal, WriteUnreal};
use std::thread;
use std::thread::JoinHandle;

impl From<(&HuntingZone, &mut L2GeneralStringTable)> for HuntingZoneDat {
    fn from(value: (&HuntingZone, &mut L2GeneralStringTable)) -> Self {
        let (zone, _table) = value;

        HuntingZoneDat {
            id: zone.id.into(),
            zone_type: zone.zone_type.to_u32().unwrap(),
            recommended_level_min: zone.lvl_min,
            recommended_level_max: zone.lvl_max,
            start_npc_loc: zone.start_npc_loc.into(),
            desc: (&zone.desc).into(),
            search_zone_id: zone.search_zone_id.into(),
            name: (&zone.name).into(),
            region_id: zone.region_id.into(),
            npc_id: zone.npc_id.into(),
            quests: zone.quests.iter().map(|v| (*v).into()).collect(),
            instant_zone_id: zone.instant_zone_id.into(),
        }
    }
}

impl Loader110 {
    pub fn serialize_hunting_zones_to_binary(&mut self) -> JoinHandle<()> {
        let hunting_zones = self
            .hunting_zones
            .values()
            .map(|v| (v, &mut self.game_data_name).into())
            .collect();

        let huntingzone_path = self
            .dat_paths
            .get(&"huntingzone-ru.dat".to_string())
            .unwrap()
            .clone();

        thread::spawn(move || {
            save_dat(
                huntingzone_path.path(),
                DatVariant::<(), HuntingZoneDat>::Array(hunting_zones),
            )
            .unwrap();

            println!("Hunting Zones Saved")
        })
    }

    pub fn load_hunting_zones(&mut self) -> Result<Vec<Log>, ()> {
        let warnings = vec![];

        let hunting_zones = deserialize_dat::<HuntingZoneDat>(
            self.dat_paths
                .get(&"huntingzone-ru.dat".to_string())
                .unwrap()
                .path(),
        )?;

        for v in hunting_zones {
            self.hunting_zones.insert(
                v.id.into(),
                HuntingZone {
                    id: v.id.into(),
                    name: v.name.to_string(),
                    desc: v.desc.to_string(),
                    zone_type: HuntingZoneType::from_u32(v.zone_type).unwrap(),
                    lvl_min: v.recommended_level_min,
                    lvl_max: v.recommended_level_max,
                    start_npc_loc: v.start_npc_loc.into(),
                    npc_id: v.npc_id.into(),
                    quests: v.quests.iter().map(|v| QuestId(*v as u32)).collect(),
                    region_id: v.region_id.into(),
                    search_zone_id: v.search_zone_id.into(),
                    instant_zone_id: v.instant_zone_id.into(),
                },
            );
        }

        Ok(warnings)
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct HuntingZoneDat {
    id: DWORD,
    zone_type: DWORD,
    recommended_level_min: DWORD,
    recommended_level_max: DWORD,
    start_npc_loc: CoordsXYZ,
    desc: ASCF,
    search_zone_id: DWORD,
    name: ASCF,
    region_id: USHORT,
    npc_id: DWORD,
    quests: Vec<USHORT>,
    instant_zone_id: DWORD,
}
