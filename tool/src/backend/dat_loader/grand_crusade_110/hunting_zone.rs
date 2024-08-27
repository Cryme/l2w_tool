use crate::backend::entity_editor::WindowParams;

use l2_rw::ue2_rw::{ASCF, DWORD, INT, SHORT, USHORT};
use l2_rw::{deserialize_dat, save_dat, DatVariant};

use l2_rw::ue2_rw::{ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal};

use crate::backend::dat_loader::grand_crusade_110::CoordsXYZ;
use crate::backend::dat_loader::L2StringTable;
use crate::backend::holder::{GameDataHolder, HolderMapOps, L2GeneralStringTable};
use crate::backend::log_holder::{Log, LogLevel};
use crate::common::QuestId;
use crate::entity::hunting_zone::{HuntingZone, HuntingZoneType, MapObject};
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
            second_id: zone.second_id,
            npc_id: zone.npc_id.into(),
            quests: zone.quests.iter().map(|v| (*v).into()).collect(),
            instant_zone_id: zone.instant_zone_id.into(),
        }
    }
}

impl GameDataHolder {
    pub fn serialize_hunting_zones_to_binary(&mut self) -> JoinHandle<Vec<Log>> {
        let mut map_objects: Vec<MiniMapRegionDat> = vec![];

        for zone in self.hunting_zone_holder.values().filter(|v| !v._deleted) {
            for item in &zone.world_map_objects {
                let item = &item.inner;

                map_objects.push(MiniMapRegionDat {
                    hunting_zone_second_id: zone.second_id,
                    icon_texture_normal: self.game_string_table.get_index(&item.icon_texture),
                    icon_texture_over: self.game_string_table.get_index(&item.icon_texture_over),
                    icon_texture_pushed: self
                        .game_string_table
                        .get_index(&item.icon_texture_pressed),
                    world_loc_x: item.world_pos[0],
                    world_loc_y: item.world_pos[1],
                    width: item.size[0],
                    height: item.size[1],
                    desc_offset_x: item.desc_offset[0],
                    desc_offset_y: item.desc_offset[1],
                    desc_font_name: self.game_string_table.get_index(&item.desc_font_name),
                    unk: item.unk1.clone(),
                })
            }
        }

        let hunting_zones = self
            .hunting_zone_holder
            .values()
            .map(|v| (v, &mut self.game_string_table).into())
            .collect();

        let huntingzone_path = self
            .dat_paths
            .get(&"huntingzone-ru.dat".to_string())
            .unwrap()
            .clone();

        let minimapregion_path = self
            .dat_paths
            .get(&"minimapregion.dat".to_string())
            .unwrap()
            .clone();

        thread::spawn(move || {
            let mut logs = vec![];

            if let Err(e) = save_dat(
                minimapregion_path.path(),
                DatVariant::<(), MiniMapRegionDat>::Array(map_objects.to_vec()),
            ) {
                logs.push(Log::from_loader_e(e));
            } else {
                logs.push(Log::from_loader_i("Mini Map Region saved"));
            }

            if let Err(e) = save_dat(
                huntingzone_path.path(),
                DatVariant::<(), HuntingZoneDat>::Array(hunting_zones),
            ) {
                logs.push(Log::from_loader_e(e));
            } else {
                logs.push(Log::from_loader_i("Hunting Zone saved"));
            }

            logs
        })
    }

    pub fn load_hunting_zones(&mut self) -> Result<Vec<Log>, ()> {
        let mut warnings = vec![];

        let hunting_zones = deserialize_dat::<HuntingZoneDat>(
            self.dat_paths
                .get(&"huntingzone-ru.dat".to_string())
                .unwrap()
                .path(),
        )?;

        let map_objects = deserialize_dat::<MiniMapRegionDat>(
            self.dat_paths
                .get(&"minimapregion.dat".to_string())
                .unwrap()
                .path(),
        )?;

        for v in hunting_zones {
            self.hunting_zone_holder.insert(
                v.id.into(),
                HuntingZone {
                    id: v.id.into(),
                    name: v.name.to_string(),
                    desc: v.desc.to_string(),
                    zone_type: HuntingZoneType::from_u32(v.zone_type)
                        .unwrap_or_else(|| panic!("unknown type: {}", v.zone_type)),
                    lvl_min: v.recommended_level_min,
                    lvl_max: v.recommended_level_max,
                    start_npc_loc: v.start_npc_loc.into(),
                    npc_id: v.npc_id.into(),
                    quests: v.quests.iter().map(|v| QuestId(*v as u32)).collect(),
                    second_id: v.second_id,
                    search_zone_id: v.search_zone_id.into(),
                    instant_zone_id: v.instant_zone_id.into(),
                    world_map_objects: vec![],
                    ..Default::default()
                },
            );
        }

        for (i, v) in map_objects.into_iter().enumerate() {
            if let Some(c) = self
                .hunting_zone_holder
                .values_mut()
                .find(|z| z.second_id == v.hunting_zone_second_id)
            {
                c.world_map_objects.push(WindowParams::new(MapObject {
                    icon_texture: self.game_string_table.get_o(&v.icon_texture_normal),
                    icon_texture_over: self.game_string_table.get_o(&v.icon_texture_over),
                    icon_texture_pressed: self.game_string_table.get_o(&v.icon_texture_pushed),
                    world_pos: [v.world_loc_x, v.world_loc_y],
                    size: [v.width, v.height],
                    desc_offset: [v.desc_offset_x, v.desc_offset_y],
                    desc_font_name: self.game_string_table.get_o(&v.desc_font_name),
                    unk1: v.unk,
                }));
            } else {
                warnings.push(Log {
                    level: LogLevel::Error,
                    producer: "Hunting Zone Loader".to_string(),
                    log: format!("Row {} in mimapregion points to\nunexisting huntingzone with secondary id {}\nRow Skipped", i, v.hunting_zone_second_id),
                })
            }
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
    second_id: USHORT,
    npc_id: DWORD,
    quests: Vec<USHORT>,
    instant_zone_id: DWORD,
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct MiniMapRegionDat {
    hunting_zone_second_id: USHORT,
    icon_texture_normal: DWORD,
    icon_texture_over: DWORD,
    icon_texture_pushed: DWORD,

    world_loc_x: INT,
    world_loc_y: INT,

    width: USHORT,
    height: USHORT,

    desc_offset_x: SHORT,
    desc_offset_y: SHORT,

    desc_font_name: DWORD,

    unk: Vec<INT>,
}
