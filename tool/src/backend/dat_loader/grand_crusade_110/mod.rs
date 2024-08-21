#![allow(
    clippy::needless_borrows_for_generic_args,
    clippy::unnecessary_to_owned
)]
mod animation_combo;
mod daily_mission;
mod hunting_zone;
mod item;
mod item_set;
mod npc;
mod quest;
mod raid_data;
mod recipe;
mod region;
mod residence;
mod skill;

use crate::backend::holder::{GameDataHolder, HolderMapOps, HolderOps, L2GeneralStringTable};
use crate::data::{Location, Position};
use crate::frontend::IS_SAVING;

use crate::backend::dat_loader::{DatLoader, L2StringTable};
use crate::backend::log_holder::Log;
use crate::entity::{CommonEntity, Entity};
use l2_rw::ue2_rw::{ASCF, BYTE, DWORD, FLOAT, STR};
use l2_rw::{deserialize_dat, save_dat, DatVariant};
use r#macro::{ReadUnreal, WriteUnreal};
use std::collections::hash_map::Keys;
use std::collections::HashMap;
use std::ops::Index;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::thread;
use strum::IntoEnumIterator;
use walkdir::DirEntry;

use crate::log_multiple;
use l2_rw::ue2_rw::{ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal};

#[derive(Default, Clone)]
pub struct L2SkillStringTable {
    next_index: u32,
    inner: HashMap<u32, String>,
    reverse_map: HashMap<String, u32>,
}

impl L2StringTable for L2SkillStringTable {
    fn keys(&self) -> Keys<u32, String> {
        self.inner.keys()
    }
    fn get(&self, key: &u32) -> Option<&String> {
        self.inner.get(key)
    }
    fn get_o(&self, key: &u32) -> String {
        self.inner
            .get(key)
            .cloned()
            .unwrap_or_else(|| format!("NameNotFound[{}]", key))
    }
    fn from_vec(values: Vec<String>) -> Self {
        let mut s = Self::default();

        for v in values {
            s.add(v);
        }

        s
    }

    fn get_index(&mut self, value: &str) -> u32 {
        if let Some(i) = self.reverse_map.get(value) {
            *i
        } else {
            self.add(value.to_string())
        }
    }

    fn add(&mut self, value: String) -> u32 {
        self.reverse_map.insert(value.clone(), self.next_index);
        self.inner.insert(self.next_index, value);
        self.next_index += 1;

        self.next_index - 1
    }
}

impl Index<usize> for L2SkillStringTable {
    type Output = String;

    fn index(&self, index: usize) -> &Self::Output {
        self.inner.get(&(index as u32)).unwrap()
    }
}

impl Index<u32> for L2SkillStringTable {
    type Output = String;

    fn index(&self, index: u32) -> &Self::Output {
        self.inner.get(&index).unwrap()
    }
}

impl DatLoader for GameDataHolder {
    fn load_from_binary(&mut self, dat_paths: HashMap<String, DirEntry>) -> Result<Vec<Log>, ()> {
        let Some(path) = dat_paths.get(&"l2gamedataname.dat".to_string()) else {
            return Err(());
        };

        self.game_string_table = Self::load_game_data_name(path.path())?;
        self.dat_paths = dat_paths;

        let mut logs = vec![];

        logs.extend(self.load_npcs()?);
        logs.extend(self.load_npc_strings()?);

        logs.extend(self.load_items()?);
        self.refill_all_items();

        logs.extend(self.load_hunting_zones()?);
        logs.extend(self.load_quests()?);
        logs.extend(self.load_skills()?);
        logs.extend(self.load_item_sets()?);
        logs.extend(self.load_recipes()?);
        logs.extend(self.load_hunting_zones()?);
        logs.extend(self.load_regions()?);
        logs.extend(self.load_raid_data()?);
        logs.extend(self.load_daily_missions()?);
        logs.extend(self.load_animation_combo()?);
        logs.extend(self.load_residences()?);

        let mut log = "Dats loaded".to_string();
        log.push_str(&format!("\nNpcs: {}", self.npc_holder.len()));
        log.push_str(&format!("\nNpc Strings: {}", self.npc_strings.len()));
        log.push_str(&format!("\nQuests: {}", self.quest_holder.len()));
        log.push_str(&format!("\nSkills: {}", self.skill_holder.len()));
        log.push_str(&format!("\nItems: {}", self.item_holder.len()));
        log.push_str(&format!("\n\t Weapons: {}", self.weapon_holder.len()));
        log.push_str(&format!("\n\t EtcItems: {}", self.etc_item_holder.len()));
        log.push_str(&format!("\n\t Armor: {}", self.armor_holder.len()));
        log.push_str(&format!("\nItem Sets: {}", self.item_set_holder.len()));
        log.push_str(&format!("\nRecipes: {}", self.recipe_holder.len()));
        log.push_str(&format!(
            "\nHunting Zones: {}",
            self.hunting_zone_holder.len()
        ));
        log.push_str(&format!("\nRegions: {}", self.region_holder.len()));
        log.push_str(&format!("\nRaids: {}", self.raid_info_holder.len()));
        log.push_str(&format!(
            "\nDaily Missions: {}",
            self.daily_mission_holder.len()
        ));
        log.push_str(&format!(
            "\nAnimation Combo: {}",
            self.animation_combo_holder.len()
        ));
        log.push_str(&format!("\nResidences: {}", self.residence_holder.len()));
        log.push_str("\n======================================");

        logs.push(Log::from_loader_i(&log));

        Ok(logs)
    }

    fn save_to_binary(&mut self, ron_path: &Option<String>) -> std::io::Result<()> {
        let mut res = vec![];

        IS_SAVING.store(true, Ordering::Relaxed);

        if let Some(path) = ron_path {
            if let Err(e) = self.save_to_ron(path, false) {
                res.push(Log::from_loader_e(e.to_string()));
            }
        }

        let skills_handle = if self.skill_holder.was_changed() {
            Some(self.serialize_skills_to_binary())
        } else {
            None
        };
        let quest_handle = if self.quest_holder.was_changed() {
            Some(self.serialize_quests_to_binary())
        } else {
            None
        };

        let npcs_handle = if self.npc_holder.was_changed() {
            Some(self.serialize_npcs_to_binary())
        } else {
            None
        };

        let items_handle = if self.weapon_holder.was_changed()
            || self.etc_item_holder.was_changed()
            || self.armor_holder.was_changed()
        {
            Some(self.serialize_items_to_binary())
        } else {
            None
        };

        let item_sets_handle = if self.item_set_holder.was_changed() {
            Some(self.serialize_item_sets_to_binary())
        } else {
            None
        };

        let recipes_handle = if self.recipe_holder.was_changed() {
            Some(self.serialize_recipes_to_binary())
        } else {
            None
        };

        let hunting_zones_handle = if self.hunting_zone_holder.was_changed() {
            Some(self.serialize_hunting_zones_to_binary())
        } else {
            None
        };

        let regions_handle = if self.region_holder.was_changed() {
            Some(self.serialize_regions_to_binary())
        } else {
            None
        };

        let raid_info_handle = if self.raid_info_holder.was_changed() {
            Some(self.serialize_raid_data_to_binary())
        } else {
            None
        };

        let daily_missions_handle = if self.daily_mission_holder.was_changed() {
            Some(self.serialize_daily_missions_to_binary())
        } else {
            None
        };

        let animations_combo_handle = if self.animation_combo_holder.was_changed() {
            Some(self.serialize_animation_combo_to_binary())
        } else {
            None
        };

        let residences_handle = if self.residence_holder.was_changed() {
            Some(self.serialize_residence_to_binary())
        } else {
            None
        };

        let gdn_changed = self.game_string_table.was_changed;

        let l2_game_data_name_values = self.game_string_table.to_vec();
        let l2_game_data_name = self
            .dat_paths
            .get(&"l2gamedataname.dat".to_string())
            .unwrap()
            .clone();

        thread::spawn(move || {
            let gdn_handel = if gdn_changed {
                Some(thread::spawn(move || {
                    if let Err(e) = save_dat(
                        l2_game_data_name.path(),
                        DatVariant::<(), String>::Array(l2_game_data_name_values),
                    ) {
                        Log::from_loader_e(&format!("{e:?}"))
                    } else {
                        Log::from_loader_i("Game Data Name saved")
                    }
                }))
            } else {
                None
            };

            if let Some(v) = gdn_handel {
                res.push(v.join().unwrap());
            }

            if let Some(v) = skills_handle {
                res.extend(v.join().unwrap());
            }

            if let Some(v) = quest_handle {
                res.extend(v.join().unwrap());
            }

            if let Some(v) = npcs_handle {
                res.extend(v.join().unwrap());
            }

            if let Some(v) = items_handle {
                res.extend(v.join().unwrap());
            }

            if let Some(v) = item_sets_handle {
                res.push(v.join().unwrap());
            }

            if let Some(v) = recipes_handle {
                res.push(v.join().unwrap());
            }

            if let Some(v) = hunting_zones_handle {
                res.extend(v.join().unwrap());
            }

            if let Some(v) = regions_handle {
                res.push(v.join().unwrap());
            }

            if let Some(v) = raid_info_handle {
                res.push(v.join().unwrap());
            }

            if let Some(v) = daily_missions_handle {
                res.push(v.join().unwrap());
            }

            if let Some(v) = animations_combo_handle {
                res.push(v.join().unwrap());
            }

            if let Some(v) = residences_handle {
                res.push(v.join().unwrap());
            }

            res.push(Log::from_loader_i("Binaries Saved"));

            log_multiple(res);

            IS_SAVING.store(false, Ordering::Relaxed);
        });

        Ok(())
    }

    fn save_to_ron(&self, folder_path: &str, all: bool) -> std::io::Result<()> {
        for e in Entity::iter() {
            if all || self[e].was_changed() {
                let _ = self[e].save_to_ron(Path::new(folder_path).join(format!("{e}.ron")));
            }
        }

        if all || self.game_string_table.was_changed {
            let _ = self
                .game_string_table
                .save_to_ron(Path::new(folder_path).join("L2GameDataName.ron"));
            let _ = self
                .npc_strings
                .save_to_ron(Path::new(folder_path).join("NpcStrings.ron"));
        }

        Ok(())
    }
}

impl GameDataHolder {
    /**Returns cloned String from `game data name`
     */
    fn gdns_cloned(&self, index: &u32) -> String {
        self.game_string_table[index].clone()
    }

    /**Returns Vector of cloned Strings from `game data name`
     */
    fn vec_gdns_cloned(&self, indexes: &[u32]) -> Vec<String> {
        indexes
            .iter()
            .map(|v| self.game_string_table[v].clone())
            .collect()
    }

    fn load_game_data_name(path: &Path) -> Result<L2GeneralStringTable, ()> {
        match deserialize_dat(path) {
            Ok(r) => Ok(L2GeneralStringTable::from_vec(r)),
            Err(e) => Err(e),
        }
    }

    fn load_npc_strings(&mut self) -> Result<Vec<Log>, ()> {
        let vals = deserialize_dat::<NpcStringDat>(
            self.dat_paths
                .get(&"npcstring-ru.dat".to_string())
                .unwrap()
                .path(),
        )?;

        for v in vals {
            self.npc_strings.insert(v.id, v.value.to_string());
        }

        Ok(vec![])
    }

    fn refill_all_items(&mut self) {
        self.item_holder.clear();

        self.item_holder
            .extend(self.weapon_holder.values().map(|v| (v.id(), v.into())));
        self.item_holder
            .extend(self.etc_item_holder.values().map(|v| (v.id(), v.into())));
        self.item_holder
            .extend(self.armor_holder.values().map(|v| (v.id(), v.into())));
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct L2GameDataNameDat {
    value: STR,
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct NpcStringDat {
    id: DWORD,
    value: ASCF,
}

#[derive(Debug, Copy, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
pub struct CoordsXYZ {
    pub(crate) x: FLOAT,
    pub(crate) y: FLOAT,
    pub(crate) z: FLOAT,
}

impl From<CoordsXYZ> for Position {
    fn from(value: CoordsXYZ) -> Self {
        Position {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<Position> for CoordsXYZ {
    fn from(value: Position) -> Self {
        CoordsXYZ {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<CoordsXYZ> for Location {
    fn from(value: CoordsXYZ) -> Self {
        Location {
            x: value.x as i32,
            y: value.y as i32,
            z: value.z as i32,
        }
    }
}

impl From<Location> for CoordsXYZ {
    fn from(value: Location) -> Self {
        CoordsXYZ {
            x: value.x as f32,
            y: value.y as f32,
            z: value.z as f32,
        }
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
pub struct Color {
    pub b: BYTE,
    pub g: BYTE,
    pub r: BYTE,
    pub a: BYTE,
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
pub struct Collision {
    pub radius_1: FLOAT,
    pub radius_2: FLOAT,
    pub height_1: FLOAT,
    pub height_2: FLOAT,
}
