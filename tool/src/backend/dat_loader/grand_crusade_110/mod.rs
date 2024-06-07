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
mod skill;

use crate::backend::holder::{
    ChroniclesProtocol, FDHashMap, FHashMap, GameDataHolder, HolderMapOps, L2GeneralStringTable,
};
use crate::data::{
    AnimationComboId, DailyMissionId, HuntingZoneId, ItemId, ItemSetId, Location, NpcId, Position,
    QuestId, RaidInfoId, RecipeId, RegionId, SkillId,
};
use crate::entity::hunting_zone::HuntingZone;
use crate::entity::item::Item;
use crate::entity::npc::Npc;
use crate::entity::quest::Quest;
use crate::entity::skill::Skill;
use crate::frontend::IS_SAVING;

use crate::backend::dat_loader::{DatLoader, L2StringTable};
use crate::backend::log_holder::Log;
use crate::entity::item::armor::Armor;
use crate::entity::item::etc_item::EtcItem;
use crate::entity::item::weapon::Weapon;
use crate::entity::item_set::ItemSet;
use crate::entity::recipe::Recipe;
use crate::entity::CommonEntity;
use l2_rw::ue2_rw::{ASCF, BYTE, DWORD, FLOAT, STR};
use l2_rw::{deserialize_dat, save_dat, DatVariant};
use r#macro::{ReadUnreal, WriteUnreal};
use std::collections::hash_map::Keys;
use std::collections::HashMap;
use std::ops::Index;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::thread;
use walkdir::DirEntry;

use crate::entity::animation_combo::AnimationCombo;
use crate::entity::daily_mission::DailyMission;
use crate::entity::raid_info::RaidInfo;
use crate::entity::region::Region;
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

#[derive(Default)]
pub struct Loader110 {
    game_data_name: L2GeneralStringTable,
    dat_paths: HashMap<String, DirEntry>,

    quests: FHashMap<QuestId, Quest>,
    skills: FHashMap<SkillId, Skill>,
    npcs: FHashMap<NpcId, Npc>,

    all_items: HashMap<ItemId, Item>,
    weapons: FHashMap<ItemId, Weapon>,
    armor: FHashMap<ItemId, Armor>,
    etc_items: FHashMap<ItemId, EtcItem>,

    item_sets: FHashMap<ItemSetId, ItemSet>,
    recipes: FHashMap<RecipeId, Recipe>,
    hunting_zones: FHashMap<HuntingZoneId, HuntingZone>,
    regions: FHashMap<RegionId, Region>,

    raid_info: FHashMap<RaidInfoId, RaidInfo>,
    daily_missions: FHashMap<DailyMissionId, DailyMission>,
    animation_combo: FDHashMap<AnimationComboId, AnimationCombo>,

    npc_strings: FHashMap<u32, String>,
}

impl DatLoader for Loader110 {
    fn load(&mut self, dat_paths: HashMap<String, DirEntry>) -> Result<Vec<Log>, ()> {
        let Some(path) = dat_paths.get(&"l2gamedataname.dat".to_string()) else {
            return Err(());
        };

        self.game_data_name = Self::load_game_data_name(path.path())?;
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

        let mut log = "Dats loaded".to_string();
        log.push_str(&format!("\nNpcs: {}", self.npcs.len()));
        log.push_str(&format!("\nNpc Strings: {}", self.npc_strings.len()));
        log.push_str(&format!("\nQuests: {}", self.quests.len()));
        log.push_str(&format!("\nSkills: {}", self.skills.len()));
        log.push_str(&format!("\nItems: {}", self.all_items.len()));
        log.push_str(&format!("\n\t Weapons: {}", self.weapons.len()));
        log.push_str(&format!("\n\t EtcItems: {}", self.etc_items.len()));
        log.push_str(&format!("\n\t Armor: {}", self.armor.len()));
        log.push_str(&format!("\nItem Sets: {}", self.item_sets.len()));
        log.push_str(&format!("\nRecipes: {}", self.recipes.len()));
        log.push_str(&format!("\nHunting Zones: {}", self.hunting_zones.len()));
        log.push_str(&format!("\nRegions: {}", self.regions.len()));
        log.push_str(&format!("\nRaids: {}", self.raid_info.len()));
        log.push_str(&format!("\nDaily Missions: {}", self.daily_missions.len()));
        log.push_str(&format!(
            "\nAnimation Combo: {}",
            self.animation_combo.len()
        ));
        log.push_str("\n======================================");

        logs.push(Log::from_loader_i(&log));

        Ok(logs)
    }

    fn from_holder(game_data_holder: &GameDataHolder) -> Self {
        let items_changed = game_data_holder.armor_holder.was_changed()
            || game_data_holder.etc_item_holder.was_changed()
            || game_data_holder.weapon_holder.was_changed();

        Self {
            dat_paths: game_data_holder.initial_dat_paths.clone(),

            quests: game_data_holder.quest_holder.changed_or_empty(),

            game_data_name: game_data_holder.game_string_table.clone(),

            skills: game_data_holder.skill_holder.changed_or_empty(),

            npcs: game_data_holder.npc_holder.changed_or_empty(),

            npc_strings: game_data_holder.npc_strings.changed_or_empty(),

            all_items: Default::default(),

            weapons: if items_changed {
                game_data_holder.weapon_holder.clone()
            } else {
                FHashMap::new()
            },
            etc_items: if items_changed {
                game_data_holder.etc_item_holder.clone()
            } else {
                FHashMap::new()
            },
            armor: if items_changed {
                game_data_holder.armor_holder.clone()
            } else {
                FHashMap::new()
            },

            item_sets: game_data_holder.item_set_holder.changed_or_empty(),
            recipes: game_data_holder.recipe_holder.changed_or_empty(),

            hunting_zones: game_data_holder.hunting_zone_holder.changed_or_empty(),
            regions: game_data_holder.region_holder.changed_or_empty(),
            raid_info: game_data_holder.raid_info_holder.changed_or_empty(),
            daily_missions: game_data_holder.daily_mission_holder.changed_or_empty(),
            animation_combo: game_data_holder.animation_combo_holder.changed_or_empty(),
        }
    }

    fn to_holder(self) -> GameDataHolder {
        let mut r = GameDataHolder {
            protocol_version: ChroniclesProtocol::GrandCrusade110,
            initial_dat_paths: self.dat_paths,
            npc_holder: self.npcs,
            npc_strings: self.npc_strings,
            item_holder: self.all_items,
            quest_holder: self.quests,
            skill_holder: self.skills,
            weapon_holder: self.weapons,
            armor_holder: self.armor,
            etc_item_holder: self.etc_items,
            item_set_holder: self.item_sets,
            recipe_holder: self.recipes,
            hunting_zone_holder: self.hunting_zones,
            region_holder: self.regions,
            raid_info_holder: self.raid_info,
            daily_mission_holder: self.daily_missions,
            animation_combo_holder: self.animation_combo,

            game_string_table: self.game_data_name,
        };

        r.npc_holder.set_changed(false);
        r.npc_strings.set_changed(false);
        r.quest_holder.set_changed(false);
        r.skill_holder.set_changed(false);
        r.weapon_holder.set_changed(false);
        r.armor_holder.set_changed(false);
        r.etc_item_holder.set_changed(false);
        r.item_set_holder.set_changed(false);
        r.recipe_holder.set_changed(false);
        r.hunting_zone_holder.set_changed(false);
        r.region_holder.set_changed(false);
        r.raid_info_holder.set_changed(false);
        r.daily_mission_holder.set_changed(false);
        r.animation_combo_holder.set_changed(false);

        r
    }

    fn serialize_to_binary(&mut self) -> std::io::Result<()> {
        let mut res = vec![];

        IS_SAVING.store(true, Ordering::Relaxed);

        let skills_handle = if self.skills.was_changed() {
            Some(self.serialize_skills_to_binary())
        } else {
            None
        };
        let quest_handle = if self.quests.was_changed() {
            Some(self.serialize_quests_to_binary())
        } else {
            None
        };

        let npcs_handle = if self.npcs.was_changed() {
            Some(self.serialize_npcs_to_binary())
        } else {
            None
        };

        let items_handle = if self.weapons.was_changed()
            || self.etc_items.was_changed()
            || self.armor.was_changed()
        {
            Some(self.serialize_items_to_binary())
        } else {
            None
        };

        let item_sets_handle = if self.item_sets.was_changed() {
            Some(self.serialize_item_sets_to_binary())
        } else {
            None
        };

        let recipes_handle = if self.recipes.was_changed() {
            Some(self.serialize_recipes_to_binary())
        } else {
            None
        };

        let hunting_zones_handle = if self.hunting_zones.was_changed() {
            Some(self.serialize_hunting_zones_to_binary())
        } else {
            None
        };

        let regions_handle = if self.regions.was_changed() {
            Some(self.serialize_regions_to_binary())
        } else {
            None
        };

        let raid_info_handle = if self.raid_info.was_changed() {
            Some(self.serialize_raid_data_to_binary())
        } else {
            None
        };

        let daily_missions_handle = if self.daily_missions.was_changed() {
            Some(self.serialize_daily_missions_to_binary())
        } else {
            None
        };

        let animations_combo_handle = if self.animation_combo.was_changed() {
            Some(self.serialize_animation_combo_to_binary())
        } else {
            None
        };

        let gdn_changed = self.game_data_name.was_changed;

        let l2_game_data_name_values = self.game_data_name.to_vec();
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

            res.push(Log::from_loader_i("Binaries Saved"));

            log_multiple(res);

            IS_SAVING.store(false, Ordering::Relaxed);
        });

        Ok(())
    }
}

impl Loader110 {
    /**Returns cloned String from `game data name`
     */
    fn gdns_cloned(&self, index: &u32) -> String {
        self.game_data_name[index].clone()
    }

    /**Returns Vector of cloned Strings from `game data name`
     */
    fn vec_gdns_cloned(&self, indexes: &[u32]) -> Vec<String> {
        indexes
            .iter()
            .map(|v| self.game_data_name[v].clone())
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
        self.all_items.clear();

        self.all_items
            .extend(self.weapons.values().map(|v| (v.id(), v.into())));
        self.all_items
            .extend(self.etc_items.values().map(|v| (v.id(), v.into())));
        self.all_items
            .extend(self.armor.values().map(|v| (v.id(), v.into())));
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
