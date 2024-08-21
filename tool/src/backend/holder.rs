use crate::backend::dat_loader::L2StringTable;
use crate::backend::entity_editor::WindowParams;
use crate::backend::server_side::ServerDataHolder;
use crate::backend::Config;
use crate::data::{
    AnimationComboId, DailyMissionId, HuntingZoneId, ItemId, ItemSetId, NpcId, QuestId, RaidInfoId,
    RecipeId, RegionId, ResidenceId, SkillId,
};
use crate::entity::animation_combo::AnimationCombo;
use crate::entity::daily_mission::DailyMission;
use crate::entity::hunting_zone::HuntingZone;
use crate::entity::item::armor::Armor;
use crate::entity::item::etc_item::EtcItem;
use crate::entity::item::weapon::Weapon;
use crate::entity::item::Item;
use crate::entity::item_set::ItemSet;
use crate::entity::npc::Npc;
use crate::entity::quest::Quest;
use crate::entity::raid_info::RaidInfo;
use crate::entity::recipe::Recipe;
use crate::entity::region::Region;
use crate::entity::residence::Residence;
use crate::entity::skill::Skill;
use crate::entity::{CommonEntity, Entity};
use ron::ser::PrettyConfig;
use serde::Serialize;
use std::collections::hash_map::{Keys, Values, ValuesMut};
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::{Read, Write};
use std::ops::{Index, IndexMut};
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use strum::IntoEnumIterator;
use walkdir::DirEntry;

#[derive(Default)]
pub struct GameDataHolder {
    pub dat_paths: HashMap<String, DirEntry>,

    pub npc_holder: FHashMap<NpcId, Npc>,
    pub quest_holder: FHashMap<QuestId, Quest>,
    pub skill_holder: FHashMap<SkillId, Skill>,
    pub weapon_holder: FHashMap<ItemId, Weapon>,
    pub armor_holder: FHashMap<ItemId, Armor>,
    pub etc_item_holder: FHashMap<ItemId, EtcItem>,
    pub item_set_holder: FHashMap<ItemSetId, ItemSet>,
    pub recipe_holder: FHashMap<RecipeId, Recipe>,
    pub hunting_zone_holder: FHashMap<HuntingZoneId, HuntingZone>,
    pub region_holder: FHashMap<RegionId, Region>,
    pub raid_info_holder: FHashMap<RaidInfoId, RaidInfo>,
    pub daily_mission_holder: FHashMap<DailyMissionId, DailyMission>,
    pub animation_combo_holder: FDHashMap<AnimationComboId, AnimationCombo>,
    pub residence_holder: FHashMap<ResidenceId, Residence>,

    pub item_holder: HashMap<ItemId, Item>,
    pub npc_strings: FHashMap<u32, String>,
    pub game_string_table: L2GeneralStringTable,
}

impl Index<Entity> for GameDataHolder {
    type Output = dyn HolderOps;

    fn index(&self, index: Entity) -> &Self::Output {
        match index {
            Entity::Npc => &self.npc_holder,
            Entity::Quest => &self.quest_holder,
            Entity::Skill => &self.skill_holder,
            Entity::Weapon => &self.weapon_holder,
            Entity::Armor => &self.armor_holder,
            Entity::EtcItem => &self.etc_item_holder,
            Entity::ItemSet => &self.item_set_holder,
            Entity::Recipe => &self.recipe_holder,
            Entity::HuntingZone => &self.hunting_zone_holder,
            Entity::Region => &self.region_holder,
            Entity::RaidInfo => &self.raid_info_holder,
            Entity::DailyMission => &self.daily_mission_holder,
            Entity::AnimationCombo => &self.animation_combo_holder,
            Entity::Residence => &self.residence_holder,
        }
    }
}

impl IndexMut<Entity> for GameDataHolder {
    fn index_mut(&mut self, index: Entity) -> &mut Self::Output {
        match index {
            Entity::Npc => &mut self.npc_holder,
            Entity::Quest => &mut self.quest_holder,
            Entity::Skill => &mut self.skill_holder,
            Entity::Weapon => &mut self.weapon_holder,
            Entity::Armor => &mut self.armor_holder,
            Entity::EtcItem => &mut self.etc_item_holder,
            Entity::ItemSet => &mut self.item_set_holder,
            Entity::Recipe => &mut self.recipe_holder,
            Entity::HuntingZone => &mut self.hunting_zone_holder,
            Entity::Region => &mut self.region_holder,
            Entity::RaidInfo => &mut self.raid_info_holder,
            Entity::DailyMission => &mut self.daily_mission_holder,
            Entity::AnimationCombo => &mut self.animation_combo_holder,
            Entity::Residence => &mut self.residence_holder,
        }
    }
}

impl GameDataHolder {
    pub fn set_all_holders_unchanged(&mut self) {
        for e in Entity::iter() {
            self[e].set_changed(false)
        }

        self.npc_strings.set_changed(false);
        self.game_string_table.set_changed(false);
    }

    pub fn changed_entities(&self) -> Vec<Entity> {
        let mut res = vec![];

        for e in Entity::iter() {
            if self[e].was_changed() {
                res.push(e);
            }
        }

        res
    }

    pub fn validate_paths(config: &mut Config) {
        if let Some(path) = &config.system_folder_path {
            if !Path::new(path).is_dir() {
                config.system_folder_path = None
            }
        }
    }

    pub fn get_npc_name(&self, id: &NpcId) -> String {
        if let Some(npc) = self.npc_holder.get(id) {
            npc.name.clone()
        } else {
            format!("{id:?} Not Exist!")
        }
    }

    pub fn get_item_name(&self, id: &ItemId) -> String {
        if let Some(item) = self.item_holder.get(id) {
            item.name.clone()
        } else {
            format!("{id:?} Not Exist!")
        }
    }
}

//--------------------------------------------------------------------------------------------------
//                                           Holder Structs
//--------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct FHashMap<K: Hash + Eq, V> {
    was_changed: bool,
    deleted_count: u32,
    inner: HashMap<K, V>,
}

#[allow(unused)]
pub trait HolderOps {
    fn set_changed(&mut self, val: bool);
    fn was_changed(&self) -> bool;
    fn inc_deleted(&mut self);
    fn dec_deleted(&mut self);
    fn new() -> Self
    where
        Self: Sized;
    fn is_unchanged(&self) -> bool;
    fn save_to_ron_limited(&self, folder_path: &str, entity_name: &str) -> anyhow::Result<()>;
    fn save_to_ron(&self, folder_path: &str, entity_name: &str) -> anyhow::Result<()>;
}
#[allow(unused)]
pub trait HolderMapOps<K: Hash + Eq + Copy + Clone, V: Clone + CommonEntity<K>> {
    fn remove(&mut self, key: &K) -> Option<V>;
    fn values_mut(&mut self) -> ValuesMut<'_, K, V>;
    fn keys(&self) -> Keys<K, V>;
    fn values(&self) -> Values<K, V>;
    fn get(&self, key: &K) -> Option<&V>;
    fn get_mut(&mut self, key: &K) -> Option<&mut V>;
    fn insert(&mut self, key: K, val: V) -> Option<V>;
    fn len(&self) -> usize;
}

impl<K: Hash + Eq + Copy + Clone + Ord, V: Clone + CommonEntity<K> + Serialize> HolderOps
    for FHashMap<K, V>
{
    fn set_changed(&mut self, val: bool) {
        self.was_changed = val;
    }

    fn was_changed(&self) -> bool {
        self.was_changed || self.deleted_count != 0
    }

    fn inc_deleted(&mut self) {
        self.deleted_count += 1;
    }

    fn dec_deleted(&mut self) {
        if self.deleted_count == 0 {
            return;
        }

        self.deleted_count -= 1;
    }

    fn new() -> FHashMap<K, V> {
        Self {
            was_changed: false,
            deleted_count: 0,
            inner: HashMap::new(),
        }
    }

    fn is_unchanged(&self) -> bool {
        !self.was_changed
    }

    fn save_to_ron_limited(&self, folder_path: &str, entity_name: &str) -> anyhow::Result<()> {
        save_to_ron_limited(self, folder_path, entity_name)
    }

    fn save_to_ron(&self, folder_path: &str, entity_name: &str) -> anyhow::Result<()> {
        save_to_ron(self, folder_path, entity_name)
    }
}

impl<K: Hash + Eq + Copy + Clone + Ord, V: Clone + CommonEntity<K>> HolderMapOps<K, V>
    for FHashMap<K, V>
{
    fn remove(&mut self, key: &K) -> Option<V> {
        self.inner.remove(key)
    }

    fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        self.inner.values_mut()
    }

    fn keys(&self) -> Keys<K, V> {
        self.inner.keys()
    }

    fn values(&self) -> Values<K, V> {
        self.inner.values()
    }

    fn get(&self, key: &K) -> Option<&V> {
        self.inner.get(key)
    }
    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.inner.get_mut(key)
    }

    fn insert(&mut self, key: K, val: V) -> Option<V> {
        self.was_changed = true;
        self.inner.insert(key, val)
    }

    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K: Hash + Eq, V> Default for FHashMap<K, V> {
    fn default() -> Self {
        Self {
            was_changed: false,
            deleted_count: 0,
            inner: HashMap::new(),
        }
    }
}

#[derive(Clone)]
pub struct FDHashMap<K: Hash + Eq, V> {
    was_changed: bool,
    deleted_count: u32,
    inner: HashMap<K, V>,
    inner_double: HashMap<String, K>,
}

impl<K: Hash + Eq + Copy + Clone, V: Clone + CommonEntity<K>> FDHashMap<K, V> {
    pub fn get_by_secondary(&self, key: &String) -> Option<&V> {
        if let Some(k) = self.inner_double.get(key) {
            self.inner.get(k)
        } else {
            None
        }
    }
}

impl<K: Hash + Eq + Copy + Clone + Ord, V: Clone + CommonEntity<K> + Serialize> HolderOps
    for FDHashMap<K, V>
{
    fn set_changed(&mut self, val: bool) {
        self.was_changed = val;
    }

    fn was_changed(&self) -> bool {
        self.was_changed || self.deleted_count != 0
    }

    fn inc_deleted(&mut self) {
        self.deleted_count += 1;
    }

    fn dec_deleted(&mut self) {
        if self.deleted_count == 0 {
            return;
        }

        self.deleted_count -= 1;
    }

    fn new() -> FDHashMap<K, V> {
        Self {
            was_changed: false,
            deleted_count: 0,
            inner: HashMap::new(),
            inner_double: HashMap::new(),
        }
    }

    fn is_unchanged(&self) -> bool {
        !self.was_changed
    }

    fn save_to_ron_limited(&self, folder_path: &str, entity_name: &str) -> anyhow::Result<()> {
        save_to_ron_limited(self, folder_path, entity_name)
    }

    fn save_to_ron(&self, folder_path: &str, entity_name: &str) -> anyhow::Result<()> {
        save_to_ron_limited(self, folder_path, entity_name)
    }
}

fn save_to_ron<
    K: Hash + Eq + Copy + Clone + Ord,
    V: Clone + CommonEntity<K> + Serialize,
    T: HolderMapOps<K, V>,
>(
    holder: &T,
    folder_path: &str,
    entity_name: &str,
) -> anyhow::Result<()> {
    let mut keys: Vec<_> = holder.keys().collect();

    keys.sort();

    let mut file = File::create(format!("{folder_path}/{entity_name}.ron"))?;

    for key in keys {
        file.write_all(
            &ron::ser::to_string_pretty(
                holder.get(key).unwrap(),
                PrettyConfig::default().struct_names(true),
            )
            .unwrap()
            .into_bytes(),
        )?;
        file.write_all(b"\n")?;
    }

    Ok(())
}

fn save_to_ron_limited<
    K: Hash + Eq + Copy + Clone + Ord,
    V: Clone + CommonEntity<K> + Serialize,
    T: HolderMapOps<K, V>,
>(
    holder: &T,
    folder_path: &str,
    entity_name: &str,
) -> anyhow::Result<()> {
    const MAX_ENTITIES_IN_ONE_FILE: usize = 5000;

    let mut keys: Vec<_> = holder.keys().collect();

    keys.sort();

    let mut ct = 0;
    let mut iteration = 0;

    let mut file = File::create(format!(
        "{folder_path}/{entity_name}[0-{}].ron",
        MAX_ENTITIES_IN_ONE_FILE - 1
    ))?;

    for key in keys {
        file.write_all(
            &ron::ser::to_string_pretty(
                holder.get(key).unwrap(),
                PrettyConfig::default().struct_names(true),
            )
            .unwrap()
            .into_bytes(),
        )?;

        ct += 1;

        if ct == MAX_ENTITIES_IN_ONE_FILE {
            file.flush()?;
            iteration += 1;

            file = File::create(format!(
                "{folder_path}/{entity_name}[{}-{}].ron",
                iteration * MAX_ENTITIES_IN_ONE_FILE,
                (iteration + 1) * MAX_ENTITIES_IN_ONE_FILE - 1
            ))?;

            ct = 0;
        } else {
            file.write_all(b"\n")?;
        }
    }

    Ok(())
}

impl<K: Hash + Eq + Copy + Clone, V: Clone + CommonEntity<K>> HolderMapOps<K, V>
    for FDHashMap<K, V>
{
    fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(val) = self
            .inner_double
            .iter()
            .find(|v| v.1 == key)
            .map(|v| v.0.clone())
        {
            self.inner_double.remove(&val);
        }

        let v = self.inner.remove(key);

        if let Some(vv) = &v {
            self.inner_double.remove(&vv.name());
        }

        v
    }

    fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        self.inner.values_mut()
    }

    fn keys(&self) -> Keys<K, V> {
        self.inner.keys()
    }

    fn values(&self) -> Values<K, V> {
        self.inner.values()
    }

    fn get(&self, key: &K) -> Option<&V> {
        self.inner.get(key)
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.inner.get_mut(key)
    }

    fn insert(&mut self, key: K, val: V) -> Option<V> {
        self.was_changed = true;
        self.inner_double.insert(val.name(), key);
        self.inner.insert(key, val)
    }

    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K: Hash + Eq, V> Default for FDHashMap<K, V> {
    fn default() -> Self {
        Self {
            was_changed: false,
            deleted_count: 0,
            inner: HashMap::new(),
            inner_double: HashMap::new(),
        }
    }
}

#[derive(Default, Clone)]
pub struct L2GeneralStringTable {
    pub(crate) was_changed: bool,
    next_index: u32,
    inner: HashMap<u32, String>,
    reverse_map: HashMap<String, u32>,
}

impl L2GeneralStringTable {
    pub fn save_to_ron(&self, path: PathBuf) -> anyhow::Result<()> {
        let mut keys: Vec<_> = self.inner.keys().collect();

        keys.sort();

        let mut file = File::create(path)?;

        for key in keys {
            file.write_all(self.inner.get(key).unwrap().as_bytes())?;

            file.write_all(b"\n")?;
        }

        Ok(())
    }

    pub fn set_changed(&mut self, val: bool) {
        self.was_changed = val;
    }

    pub(crate) fn to_vec(&self) -> Vec<String> {
        let mut k: Vec<_> = self.keys().collect();
        k.sort();

        let mut res = Vec::with_capacity(k.len());

        for key in k {
            res.push(self.inner.get(key).unwrap().clone());
        }

        res
    }
}

impl L2StringTable for L2GeneralStringTable {
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
            .unwrap_or_else(|| format!("StringNotFound[{}]", key))
    }

    fn from_vec(values: Vec<String>) -> Self {
        let mut s = Self::default();

        for v in values {
            s.add(v);
        }

        s
    }

    fn get_index(&mut self, mut value: &str) -> u32 {
        const NONE_STR: &str = "None";

        if value.is_empty() {
            value = NONE_STR
        }

        if let Some(i) = self.reverse_map.get(&value.to_lowercase()) {
            *i
        } else {
            self.was_changed = true;
            self.add(value.to_string())
        }
    }

    fn add(&mut self, value: String) -> u32 {
        self.reverse_map
            .insert(value.to_lowercase(), self.next_index);
        self.inner.insert(self.next_index, value);
        self.next_index += 1;

        self.next_index - 1
    }
}

impl Index<usize> for L2GeneralStringTable {
    type Output = String;

    fn index(&self, index: usize) -> &Self::Output {
        self.inner.get(&(index as u32)).unwrap()
    }
}

impl Index<u32> for L2GeneralStringTable {
    type Output = String;

    fn index(&self, index: u32) -> &Self::Output {
        self.inner.get(&index).unwrap()
    }
}

impl Index<&u32> for L2GeneralStringTable {
    type Output = String;

    fn index(&self, index: &u32) -> &Self::Output {
        self.inner.get(index).unwrap()
    }
}

pub struct DataHolder {
    pub game_data_holder: GameDataHolder,
    pub server_data_holder: ServerDataHolder,
}

impl DataHolder {
    pub fn set_java_class(&mut self, quest: &mut Quest) {
        if let Some(v) = self.server_data_holder.quest_java_classes.get(&quest.id) {
            let mut class = "".to_string();

            File::open(v.path())
                .unwrap()
                .read_to_string(&mut class)
                .unwrap();

            quest.java_class = Some(WindowParams {
                inner: class,
                initial_id: (),
                opened: false,
                action: RwLock::new(()),
                params: (),
            });
        } else {
            quest.java_class = Some(WindowParams {
                inner: self
                    .server_data_holder
                    .generate_java_template(quest, &self.game_data_holder),
                initial_id: (),
                opened: false,
                action: RwLock::new(()),
                params: (),
            });
        }
    }
}
