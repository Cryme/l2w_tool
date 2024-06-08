use crate::backend::log_holder::Log;

use l2_rw::ue2_rw::{ASCF, DWORD, FLOAT, INT, SHORT, USHORT};
use l2_rw::{deserialize_dat, save_dat, DatVariant};

use l2_rw::ue2_rw::{ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal};

use crate::backend::dat_loader::L2StringTable;
use crate::backend::holder::{GameDataHolder, HolderMapOps, L2GeneralStringTable};
use crate::entity::region::{Continent, MapInfo, Region};
use num_traits::{FromPrimitive, ToPrimitive};
use r#macro::{ReadUnreal, WriteUnreal};
use std::thread;
use std::thread::JoinHandle;

impl From<(&Region, &mut L2GeneralStringTable, &MapInfo)> for ZoneNameDat {
    fn from(value: (&Region, &mut L2GeneralStringTable, &MapInfo)) -> Self {
        let (region, table, default_map_info) = value;

        let map_info = if let Some(v) = &region.map_info {
            v
        } else {
            default_map_info
        };

        let button = if let Some(v) = &map_info.button_pos {
            v
        } else {
            &[-1, -1]
        };

        ZoneNameDat {
            id: region.id.into(),
            map_square_x: region.world_map_square[0],
            map_square_y: region.world_map_square[1],
            z_max: region.z_range[0],
            z_min: region.z_range[1],
            name: (&region.name).into(),
            town_button_loc_x: button[0],
            town_button_loc_y: button[1],
            town_map_x: map_info.pos[0],
            town_map_y: map_info.pos[1],
            town_map_width: map_info.size[0],
            town_map_height: map_info.size[1],
            town_map_scale: map_info.scale,
            town_map_texture: table.get_index(&map_info.texture),
            color: region.color_code,
            continent: region.continent.to_u16().unwrap(),
            current_layer: region.current_layer,
            total_layers: region.total_layers,
            town_center_x: map_info.center[0],
            town_center_y: map_info.center[1],
        }
    }
}

impl GameDataHolder {
    pub fn serialize_regions_to_binary(&mut self) -> JoinHandle<Log> {
        let mut zonenames: Vec<&Region> = self
            .region_holder
            .values()
            .filter(|v| !v._deleted)
            .collect();

        zonenames.sort_by(|a, b| a.id.0.cmp(&b.id.0));

        let none_map_info = MapInfo {
            button_pos: None,
            pos: [-1, -1],
            size: [u16::MAX, u16::MAX],
            center: [-1, -1],
            scale: 0.0,
            texture: String::from("None"),
        };

        let zonenames = zonenames
            .iter()
            .map(|v| (*v, &mut self.game_string_table, &none_map_info).into())
            .collect();

        let zonename_path = self
            .dat_paths
            .get(&"zonename-ru.dat".to_string())
            .unwrap()
            .clone();

        thread::spawn(move || {
            if let Err(e) = save_dat(
                zonename_path.path(),
                DatVariant::<(), ZoneNameDat>::Array(zonenames),
            ) {
                Log::from_loader_e(e)
            } else {
                Log::from_loader_i("Mini Map Region saved")
            }
        })
    }

    pub fn load_regions(&mut self) -> Result<Vec<Log>, ()> {
        let warnings = vec![];

        let zonename = deserialize_dat::<ZoneNameDat>(
            self.dat_paths
                .get(&"zonename-ru.dat".to_string())
                .unwrap()
                .path(),
        )?;

        for v in zonename {
            let map_texture = self.game_string_table.get_o(&v.town_map_texture);

            let map_info = if map_texture.to_lowercase() == "none" {
                None
            } else {
                Some(MapInfo {
                    button_pos: if v.town_button_loc_x != -1 && v.town_button_loc_y != -1 {
                        Some([v.town_button_loc_x, v.town_button_loc_y])
                    } else {
                        None
                    },
                    pos: [v.town_map_x, v.town_map_y],
                    size: [v.town_map_height, v.town_map_width],
                    center: [v.town_center_x, v.town_center_y],
                    scale: v.town_map_scale,
                    texture: map_texture,
                })
            };

            self.region_holder.insert(
                v.id.into(),
                Region {
                    id: v.id.into(),
                    name: v.name.to_string(),
                    world_map_square: [v.map_square_x, v.map_square_y],
                    z_range: [v.z_max, v.z_min],
                    map_info,

                    color_code: v.color,
                    continent: Continent::from_u16(v.continent).unwrap(),
                    current_layer: v.current_layer,
                    total_layers: v.total_layers,
                    ..Default::default()
                },
            );
        }

        Ok(warnings)
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct ZoneNameDat {
    id: USHORT,
    map_square_x: USHORT,
    map_square_y: USHORT,
    z_max: FLOAT,
    z_min: FLOAT,
    name: ASCF,
    town_button_loc_x: SHORT,
    town_button_loc_y: SHORT,
    town_map_x: INT,
    town_map_y: INT,
    town_map_width: USHORT,
    town_map_height: USHORT,
    town_map_scale: FLOAT,
    town_map_texture: DWORD,
    color: USHORT,
    continent: USHORT,
    current_layer: USHORT,
    total_layers: USHORT,
    town_center_x: INT,
    town_center_y: INT,
}
