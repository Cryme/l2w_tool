use crate::backend::editor::WindowParams;
use crate::backend::server_side::ServerDataHolder;
use crate::backend::util::StringCow;
use crate::backend::{Backend, Config};
use crate::common::{
    AnimationComboId, DailyMissionId, EnsoulOptionId, HuntingZoneId, ItemId, ItemSetId, NpcId,
    QuestId, RaidInfoId, RecipeId, RegionId, ResidenceId, SkillId,
};
use crate::entity::animation_combo::AnimationCombo;
use crate::entity::daily_mission::DailyMission;
use crate::entity::ensoul_option::EnsoulOption;
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
use crate::entity::{CommonEntity, Dictionary, Entity, GameEntity};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::{Keys, Values, ValuesMut};
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::{Read, Write};
use std::ops::{Index, IndexMut};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use strum::IntoEnumIterator;
use walkdir::DirEntry;

#[derive(Clone, Serialize, Deserialize)]
pub struct DictItem<ID: Hash + Eq + Ord + Copy, T: Clone + Ord + Eq> {
    pub id: ID,
    pub item: T,

    #[serde(skip)]
    pub initial: T,
    #[serde(skip)]
    pub changed: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DictEditItem<ID: Hash + Eq + Ord + Copy, T: Clone + Ord + Eq> {
    pub id: ID,
    pub item: T,

    #[serde(skip)]
    pub previous: T,
    #[serde(skip)]
    pub initial: T,
    #[serde(skip)]
    pub changed: bool,
    #[serde(skip)]
    pub matches_initial: bool,
}

impl<ID: Hash + Eq + Ord + Copy, T: Clone + Ord + Eq + Default> DictEditItem<ID, T> {
    pub fn new(id: ID, initial: T) -> Self {
        Self {
            id,
            item: T::default(),
            previous: T::default(),

            changed: false,
            matches_initial: initial == T::default(),
            initial,
        }
    }
}

impl<ID: Hash + Eq + Ord + Copy, T: Clone + Ord + Eq> From<&DictItem<ID, T>>
    for DictEditItem<ID, T>
{
    fn from(val: &DictItem<ID, T>) -> Self {
        DictEditItem {
            id: val.id,
            item: val.item.clone(),
            previous: val.item.clone(),
            initial: val.initial.clone(),

            changed: false,
            matches_initial: !val.changed,
        }
    }
}

pub enum ChangeStatus {
    BecameChanged,
    BecameUnChanged,
    Same,
}

impl<ID: Hash + Eq + Ord + Copy, T: Clone + Ord + Eq> DictEditItem<ID, T> {
    pub fn check_changed_status(&mut self) -> ChangeStatus {
        self.matches_initial = self.initial == self.item;

        if self.changed && self.previous == self.item {
            self.changed = false;

            return ChangeStatus::BecameUnChanged;
        }

        if !self.changed && self.previous != self.item {
            self.changed = true;

            return ChangeStatus::BecameChanged;
        }

        ChangeStatus::Same
    }
}

impl<ID: Hash + Eq + Ord + Copy, T: Clone + Ord + Eq> DictItem<ID, T> {
    pub fn new(id: ID, initial: T) -> Self {
        Self {
            id,
            item: initial.clone(),
            initial,
            changed: false,
        }
    }
}

pub trait DictOps<K: Hash + Eq + Copy + Clone, V: Clone + CommonEntity<K>>:
    HolderOps + HolderMapOps<K, V>
{
}

impl DictOps<u32, DictItem<u32, String>> for FHashMap<u32, DictItem<u32, String>> {}

impl Index<Dictionary> for GameDataHolder {
    type Output = dyn DictOps<u32, DictItem<u32, String>>;

    fn index(&self, entity: Dictionary) -> &Self::Output {
        match entity {
            Dictionary::SystemStrings => &self.system_strings,
            Dictionary::NpcStrings => &self.npc_strings,
        }
    }
}

impl IndexMut<Dictionary> for GameDataHolder {
    fn index_mut(&mut self, entity: Dictionary) -> &mut Self::Output {
        match entity {
            Dictionary::SystemStrings => &mut self.system_strings,
            Dictionary::NpcStrings => &mut self.npc_strings,
        }
    }
}

impl Backend {
    pub fn apply_search(&mut self, dict: Dictionary) {
        self.editors.dictionaries[dict].apply_search()
    }

    pub fn store_dict(&mut self, dict: Dictionary) {
        let dict_editor = &mut self.editors.dictionaries[dict];
        let dict = &mut self.holders.game_data_holder[dict];

        for v in dict_editor.items_mut() {
            if !v.changed {
                continue;
            }

            v.previous.clone_from(&v.item);
            v.changed = false;

            if let Some(vv) = dict.get_mut(&v.id) {
                vv.item.clone_from(&v.item);
                vv.changed = v.item != vv.initial;

                if vv.changed {
                    dict.set_changed(true);
                }
            } else {
                dict.insert(
                    v.id,
                    DictItem {
                        id: v.id,
                        item: v.item.clone(),
                        initial: v.initial.clone(),
                        changed: true,
                    },
                );
            }
        }

        dict_editor.set_changed_count(0);

        self.check_for_unwrote_changed();
    }
}

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
    pub ensoul_option_holder: FHashMap<EnsoulOptionId, EnsoulOption>,

    pub item_holder: HashMap<ItemId, Item>,

    pub npc_strings: FHashMap<u32, DictItem<u32, String>>,
    pub system_strings: FHashMap<u32, DictItem<u32, String>>,
    pub game_string_table: L2GeneralStringTable,
}

impl Index<GameEntity> for GameDataHolder {
    type Output = dyn HolderOps;

    fn index(&self, index: GameEntity) -> &Self::Output {
        match index {
            GameEntity::Npc => &self.npc_holder,
            GameEntity::Quest => &self.quest_holder,
            GameEntity::Skill => &self.skill_holder,
            GameEntity::Weapon => &self.weapon_holder,
            GameEntity::Armor => &self.armor_holder,
            GameEntity::EtcItem => &self.etc_item_holder,
            GameEntity::ItemSet => &self.item_set_holder,
            GameEntity::Recipe => &self.recipe_holder,
            GameEntity::HuntingZone => &self.hunting_zone_holder,
            GameEntity::Region => &self.region_holder,
            GameEntity::RaidInfo => &self.raid_info_holder,
            GameEntity::DailyMission => &self.daily_mission_holder,
            GameEntity::AnimationCombo => &self.animation_combo_holder,
            GameEntity::Residence => &self.residence_holder,
            GameEntity::EnsoulOption => &self.ensoul_option_holder,
        }
    }
}

impl IndexMut<GameEntity> for GameDataHolder {
    fn index_mut(&mut self, index: GameEntity) -> &mut Self::Output {
        match index {
            GameEntity::Npc => &mut self.npc_holder,
            GameEntity::Quest => &mut self.quest_holder,
            GameEntity::Skill => &mut self.skill_holder,
            GameEntity::Weapon => &mut self.weapon_holder,
            GameEntity::Armor => &mut self.armor_holder,
            GameEntity::EtcItem => &mut self.etc_item_holder,
            GameEntity::ItemSet => &mut self.item_set_holder,
            GameEntity::Recipe => &mut self.recipe_holder,
            GameEntity::HuntingZone => &mut self.hunting_zone_holder,
            GameEntity::Region => &mut self.region_holder,
            GameEntity::RaidInfo => &mut self.raid_info_holder,
            GameEntity::DailyMission => &mut self.daily_mission_holder,
            GameEntity::AnimationCombo => &mut self.animation_combo_holder,
            GameEntity::Residence => &mut self.residence_holder,
            GameEntity::EnsoulOption => &mut self.ensoul_option_holder,
        }
    }
}

impl GameDataHolder {
    pub fn set_all_holders_unchanged(&mut self) {
        for e in GameEntity::iter() {
            self[e].set_changed(false)
        }

        for e in Dictionary::iter() {
            self[e].set_changed(false)
        }

        self.game_string_table.set_changed(false);
    }

    pub fn changed_entities(&self) -> Vec<Entity> {
        let mut res = vec![];

        for e in GameEntity::iter() {
            if self[e].was_changed() {
                res.push(e.into());
            }
        }

        for e in Dictionary::iter() {
            if self[e].was_changed() {
                res.push(e.into());
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
    fn save_to_ron_limited(
        &self,
        folder_path: &str,
        entity_name: &str,
        file_limit: u32,
    ) -> anyhow::Result<()>;
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

impl<K: Hash + Eq + Copy + Clone + Ord + Into<u32>, V: Clone + CommonEntity<K> + Serialize>
    HolderOps for FHashMap<K, V>
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

    fn save_to_ron_limited(
        &self,
        folder_path: &str,
        entity_name: &str,
        file_limit: u32,
    ) -> anyhow::Result<()> {
        save_to_ron_limited(self, folder_path, entity_name, file_limit)
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
    pub fn get_by_secondary(&self, key: &str) -> Option<&V> {
        if let Some(k) = self.inner_double.get(key) {
            self.inner.get(k)
        } else {
            None
        }
    }
}

impl<K: Hash + Eq + Copy + Clone + Ord + Into<u32>, V: Clone + CommonEntity<K> + Serialize>
    HolderOps for FDHashMap<K, V>
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

    fn save_to_ron_limited(
        &self,
        folder_path: &str,
        entity_name: &str,
        file_limit: u32,
    ) -> anyhow::Result<()> {
        save_to_ron_limited(self, folder_path, entity_name, file_limit)
    }

    fn save_to_ron(&self, folder_path: &str, entity_name: &str) -> anyhow::Result<()> {
        save_to_ron(self, folder_path, entity_name)
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
    K: Hash + Eq + Copy + Clone + Ord + Into<u32>,
    V: Clone + CommonEntity<K> + Serialize,
    T: HolderMapOps<K, V>,
>(
    holder: &T,
    folder_path: &str,
    entity_name: &str,
    file_limit: u32,
) -> anyhow::Result<()> {
    let mut keys: Vec<_> = holder.keys().collect();

    keys.sort();

    let mut max_id_in_file = file_limit - 1;

    let folder = Path::new(folder_path).join(entity_name);
    std::fs::create_dir_all(&folder)?;

    let mut file = File::create(folder.join(format!("0-{}.ron", max_id_in_file)))?;

    for key in keys {
        let v = (*key).into();

        if v > max_id_in_file {
            file.flush()?;

            max_id_in_file = v - (v % file_limit) + file_limit - 1;

            file = File::create(folder.join(format!(
                "{}-{}.ron",
                max_id_in_file - file_limit + 1,
                max_id_in_file
            )))?;
        } else {
            file.write_all(b"\n")?;
        }

        file.write_all(
            &ron::ser::to_string_pretty(
                holder.get(key).unwrap(),
                PrettyConfig::default().struct_names(true),
            )
            .unwrap()
            .into_bytes(),
        )?;
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
    inner: HashMap<u32, Arc<String>>,
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
            res.push(self.inner.get(key).unwrap().to_string());
        }

        res
    }
}

impl L2GeneralStringTable {
    pub(crate) fn keys(&self) -> Keys<u32, Arc<String>> {
        self.inner.keys()
    }

    pub(crate) fn get_o(&self, key: &u32) -> StringCow {
        StringCow::Borrowed(
            self.inner
                .get(key)
                .cloned()
                .unwrap_or_else(|| Arc::new(format!("StringNotFound[{}]", key))),
        )
    }

    pub(crate) fn from_vec(values: Vec<String>) -> Self {
        let mut s = Self::default();

        for v in values {
            s.add(v);
        }

        s
    }

    pub(crate) fn get_index(&mut self, value: &StringCow) -> u32 {
        if value.is_empty() {
            return self.get_none_index();
        }

        let lower = value.to_lowercase();

        if let Some(i) = self.reverse_map.get(&lower) {
            *i
        } else {
            self.was_changed = true;
            self.add_cow(&value, lower)
        }
    }

    pub fn get_none_index(&self) -> u32 {
        const NONE_STR: &str = "None";

        *self.reverse_map.get(NONE_STR).unwrap()
    }

    fn add_cow(&mut self, value: &StringCow, lower: String) -> u32 {
        self.reverse_map.insert(lower, self.next_index);

        self.inner.insert(
            self.next_index,
            match value {
                StringCow::Owned(v) => Arc::new(v.clone()),
                StringCow::Borrowed(v) => v.clone(),
            },
        );

        self.next_index += 1;

        self.next_index - 1
    }

    fn add(&mut self, value: String) -> u32 {
        self.reverse_map
            .insert(value.to_lowercase(), self.next_index);
        self.inner.insert(self.next_index, Arc::new(value));
        self.next_index += 1;

        self.next_index - 1
    }
}

impl Index<usize> for L2GeneralStringTable {
    type Output = Arc<String>;

    fn index(&self, index: usize) -> &Self::Output {
        self.inner.get(&(index as u32)).unwrap()
    }
}

impl Index<u32> for L2GeneralStringTable {
    type Output = Arc<String>;

    fn index(&self, index: u32) -> &Self::Output {
        self.inner.get(&index).unwrap()
    }
}

impl Index<&u32> for L2GeneralStringTable {
    type Output = Arc<String>;

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
