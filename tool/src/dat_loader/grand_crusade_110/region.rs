use crate::backend::{Log, LogLevel, WindowParams};
use crate::dat_loader::grand_crusade_110::Loader110;

use l2_rw::ue2_rw::{ASCF, DWORD, FLOAT, INT, SHORT, USHORT};
use l2_rw::{deserialize_dat, save_dat, DatVariant};

use l2_rw::ue2_rw::{ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal};

use crate::dat_loader::L2StringTable;
use crate::entity::region::{Continent, MapInfo, MapObject, Region};
use crate::holder::L2GeneralStringTable;
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

impl Loader110 {
    pub fn serialize_regions_to_binary(&mut self) -> JoinHandle<()> {
        let mut map_objects: Vec<MiniMapRegionDat> = vec![];

        for zone in self.regions.values() {
            for item in &zone.world_map_objects {
                let item = &item.inner;

                map_objects.push(MiniMapRegionDat {
                    region_id: zone.id.0.to_u16().unwrap(),
                    icon_texture_normal: self.game_data_name.get_index(&item.icon_texture),
                    icon_texture_over: self.game_data_name.get_index(&item.icon_texture_over),
                    icon_texture_pushed: self.game_data_name.get_index(&item.icon_texture_pressed),
                    world_loc_x: item.world_pos[0],
                    world_loc_y: item.world_pos[1],
                    width: item.size[0],
                    height: item.size[1],
                    desc_offset_x: item.desc_offset[0],
                    desc_offset_y: item.desc_offset[1],
                    desc_font_name: self.game_data_name.get_index(&item.desc_font_name),
                    unk: item.unk1.clone(),
                })
            }
        }

        let mut zonenames: Vec<&Region> = self.regions.values().collect();

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
            .map(|v| (*v, &mut self.game_data_name, &none_map_info).into())
            .collect();

        let zonename_path = self
            .dat_paths
            .get(&"zonename-ru.dat".to_string())
            .unwrap()
            .clone();

        let minimapregion_path = self
            .dat_paths
            .get(&"minimapregion.dat".to_string())
            .unwrap()
            .clone();

        thread::spawn(move || {
            save_dat(
                minimapregion_path.path(),
                DatVariant::<(), MiniMapRegionDat>::Array(map_objects.to_vec()),
            )
            .unwrap();
            save_dat(
                zonename_path.path(),
                DatVariant::<(), ZoneNameDat>::Array(zonenames),
            )
            .unwrap();

            println!("Regions Saved")
        })
    }

    pub fn load_regions(&mut self) -> Result<Vec<Log>, ()> {
        let mut warnings = vec![];

        let zonename = deserialize_dat::<ZoneNameDat>(
            self.dat_paths
                .get(&"zonename-ru.dat".to_string())
                .unwrap()
                .path(),
        )?;

        let map_objects = deserialize_dat::<MiniMapRegionDat>(
            self.dat_paths
                .get(&"minimapregion.dat".to_string())
                .unwrap()
                .path(),
        )?;

        for v in zonename {
            let map_texture = self.game_data_name.get_o(&v.town_map_texture);

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

            self.regions.insert(
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
                    world_map_objects: vec![],
                },
            );
        }

        let mut i = 0;
        for v in map_objects {
            if let Some(region) = self.regions.inner.get_mut(&v.region_id.into()) {
                region.world_map_objects.push(WindowParams::new(MapObject {
                    icon_texture: self.game_data_name.get_o(&v.icon_texture_normal),
                    icon_texture_over: self.game_data_name.get_o(&v.icon_texture_over),
                    icon_texture_pressed: self.game_data_name.get_o(&v.icon_texture_pushed),
                    world_pos: [v.world_loc_x, v.world_loc_y],
                    size: [v.width, v.height],
                    desc_offset: [v.desc_offset_x, v.desc_offset_y],
                    desc_font_name: self.game_data_name.get_o(&v.desc_font_name),
                    unk1: v.unk,
                }))
            } else {
                warnings.push(Log {
                    level: LogLevel::Error,
                    producer: "Region Loader".to_string(),
                    log: format!("Row {} in mimapregion points to\nunexisting Zone Name with id {}\nRow Skipped", i, v.region_id),
                })
            }

            i += 1;
        }

        Ok(warnings)
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct MiniMapRegionDat {
    region_id: USHORT,
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
