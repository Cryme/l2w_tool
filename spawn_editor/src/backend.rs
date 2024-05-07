#![allow(dead_code)]

use eframe::egui::{Pos2, Rect, Vec2};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;
use walkdir::WalkDir;

use crate::spawn_parser::parse_file;

pub const WORLD_SQUARE_SIZE: f32 = 32768.0;
pub const WORLD_SQUARE_SIZE_I32: i32 = 32768;
const WORLD_X_SQUARE_COUNT: u8 = 17;
const WORLD_Y_SQUARE_COUNT: u8 = 16;

pub const WORLD_SIZE: Vec2 = Vec2::new(
    WORLD_SQUARE_SIZE * WORLD_X_SQUARE_COUNT as f32,
    WORLD_SQUARE_SIZE * WORLD_Y_SQUARE_COUNT as f32,
);

#[inline(always)]
pub fn coord_to_map_square_raw(x: i32, y: i32) -> (u8, u8) {
    let x = (x + WORLD_SQUARE_SIZE_I32 * 9) / WORLD_SQUARE_SIZE_I32 + 11;
    let y = (y + WORLD_SQUARE_SIZE_I32 * 8) / WORLD_SQUARE_SIZE_I32 + 10;

    (x as u8, y as u8)
}

#[inline(always)]
fn coord_to_map_square(x: i32, y: i32) -> MapSquare {
    let sq = coord_to_map_square_raw(x, y);

    MapSquare { x: sq.0, y: sq.1 }
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub(crate) struct MapSquare {
    x: u8,
    y: u8,
}

trait GetMapSquare {
    fn get_map_square(&self) -> MapSquare;
}

impl GetMapSquare for PointSpawn {
    fn get_map_square(&self) -> MapSquare {
        coord_to_map_square(self.x, self.y)
    }
}

impl GetMapSquare for MapPoint {
    fn get_map_square(&self) -> MapSquare {
        coord_to_map_square(self.x, self.y)
    }
}

#[derive(Debug)]
pub struct PointSpawn {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) z: i32,
    pub(crate) heading: i32,
    pub(crate) super_point: Option<String>,
}

impl From<&PointSpawn> for Pos2 {
    fn from(value: &PointSpawn) -> Self {
        Self::new(value.x as f32, value.y as f32)
    }
}

#[derive(Debug)]
pub struct MapPoint {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) z_min: i32,
    pub(crate) z_max: i32,
}

impl From<&MapPoint> for Pos2 {
    fn from(value: &MapPoint) -> Self {
        Self::new(value.x as f32, value.y as f32)
    }
}

pub type Region = Vec<MapPoint>;

#[derive(Debug)]
pub struct TerritoryInfo {
    pub(crate) name: Option<String>,
    pub(crate) region: Region,
    pub(crate) banned_regions: Vec<Region>,
    pub(crate) z_min: i32,
    pub(crate) z_max: i32,
}

impl TerritoryInfo {
    fn is_in_zone(&self, zone: &Rect) -> bool {
        self.region.iter().any(|v| zone.contains(v.into()))
    }
}

#[derive(Debug)]
pub enum TerritoryInfoRegion {
    Named(String),
    Inlined(TerritoryInfo),
}

#[derive(Debug)]
pub struct TerritorySpawn {
    pub(crate) territory: TerritoryInfoRegion,
    pub(crate) super_point: Option<String>,
}

#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum SpawnPeriodOfDay {
    NONE,
    DAY,
    NIGHT,
}

impl SpawnPeriodOfDay {
    pub(crate) fn name(&self) -> String {
        match self {
            SpawnPeriodOfDay::NONE => "NONE".to_string(),
            SpawnPeriodOfDay::DAY => "DAY".to_string(),
            SpawnPeriodOfDay::NIGHT => "NIGHT".to_string(),
        }
    }
}

#[derive(Debug)]
pub enum Spawn {
    Point(PointSpawn),
    Territory(TerritorySpawn),
    RandomTerritory(Vec<String>),
}

impl Spawn {
    fn is_in_zone(&self, zone: &Rect, holder: &SpawnHolder) -> bool {
        match self {
            Spawn::Point(v) => zone.contains(v.into()),
            Spawn::Territory(t) => match &t.territory {
                TerritoryInfoRegion::Named(territory) => {
                    let Some(territory) = holder.territories.get(territory) else {
                        return false;
                    };

                    territory.is_in_zone(zone)
                }
                TerritoryInfoRegion::Inlined(territory) => territory.is_in_zone(zone),
            },
            Spawn::RandomTerritory(ter) => ter.iter().any(|v| {
                let Some(territory) = holder.territories.get(v) else {
                    return false;
                };

                territory.is_in_zone(zone)
            }),
        }
    }
}

#[derive(Debug)]
pub struct NpcInfo {
    pub(crate) id: u32,
    pub(crate) max: u32,
}

#[derive(Debug)]
pub struct SpawnInfo {
    pub(crate) group: String,
    pub(crate) count: u32,
    pub(crate) respawn_sec: u32,
    pub(crate) respawn_random_sec: u32,
    pub(crate) period_of_day: SpawnPeriodOfDay,
    pub(crate) npc: Vec<NpcInfo>,
    pub(crate) spawns: Vec<Spawn>,
    pub(crate) map_squares: HashSet<MapSquare>,
    pub(crate) file_name: Option<Arc<String>>,
}

impl SpawnInfo {
    fn prepare(&mut self, territories: &HashMap<String, TerritoryInfo>) {
        for spawn in &self.spawns {
            match spawn {
                Spawn::Point(v) => {
                    self.map_squares.insert(v.get_map_square());
                }
                Spawn::Territory(v) => match &v.territory {
                    TerritoryInfoRegion::Named(v) => {
                        if let Some(territory) = territories.get(v) {
                            for loc in &territory.region {
                                self.map_squares.insert(loc.get_map_square());
                            }
                        } else {
                            println!("Unknown territory: {v}");
                        }
                    }
                    TerritoryInfoRegion::Inlined(v) => {
                        for loc in &v.region {
                            self.map_squares.insert(loc.get_map_square());
                        }
                    }
                },
                Spawn::RandomTerritory(v) => {
                    for ter_name in v {
                        if let Some(territory) = territories.get(ter_name) {
                            for loc in &territory.region {
                                self.map_squares.insert(loc.get_map_square());
                            }
                        } else {
                            println!("Unknown territory: {ter_name}");
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct SpawnHolder {
    spawns: Vec<SpawnInfo>,
    pub(crate) territories: HashMap<String, TerritoryInfo>,
}

impl From<(u8, u8)> for MapSquare {
    fn from(value: (u8, u8)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

impl SpawnHolder {
    pub(crate) fn get_zone_spawns(&self, zone: &Rect) -> Vec<&SpawnInfo> {
        self.spawns
            .iter()
            .filter(|v| v.spawns.iter().any(|s| s.is_in_zone(zone, self)))
            .collect()
    }

    pub(crate) fn get_square_spawns<T: Into<MapSquare>>(&self, square: T) -> Vec<&SpawnInfo> {
        let sq = square.into();
        self.spawns
            .iter()
            .filter(|v| v.map_squares.contains(&sq))
            .collect()
    }

    pub(crate) fn get_npc_spawns(&self, npc_id: u32) -> Vec<&SpawnInfo> {
        self.spawns
            .iter()
            .filter(|v| v.npc.iter().any(|n| n.id == npc_id))
            .collect()
    }

    pub fn try_init<P: AsRef<Path>>(root_path: P) -> anyhow::Result<Self> {
        let mut spawns = Vec::new();
        let mut territories = HashMap::new();

        for path in WalkDir::new(root_path).into_iter().flatten() {
            let Ok(meta) = path.metadata() else { continue };

            if !meta.is_file() {
                continue;
            }

            let file_name = path.file_name().to_str().unwrap();
            if !file_name.ends_with(".xml") {
                continue;
            }

            parse_file(path.path(), &mut spawns, &mut territories)?;
        }

        for spawn in &mut spawns {
            spawn.prepare(&territories);
        }

        println!(
            "Loaded SpawnHolder {{\n\tTerritories: {}\n\tSpawns: {}\n}})",
            territories.len(),
            spawns.len()
        );

        Ok(Self {
            spawns,
            territories,
        })
    }
}

pub enum SpawnFilter {
    FullSquare((u8, u8)),
    InZone(Rect),
    ByNpcId(u32),
}
