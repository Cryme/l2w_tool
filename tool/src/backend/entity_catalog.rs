use crate::backend::holder::HolderMapOps;
use crate::backend::util::is_in_range;
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
use crate::entity::item_set::ItemSet;
use crate::entity::npc::Npc;
use crate::entity::quest::Quest;
use crate::entity::raid_info::RaidInfo;
use crate::entity::recipe::Recipe;
use crate::entity::region::Region;
use crate::entity::residence::Residence;
use crate::entity::skill::Skill;
use crate::entity::{CommonEntity, Entity};
use std::cmp::Ordering;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Index;
use std::str::FromStr;
use strum_macros::{Display, EnumIter};

#[derive(Copy, Clone, EnumIter, PartialEq, Eq, Display)]
pub enum FilterMode {
    All,
    Changed,
    Deleted,
}

pub struct EntityInfo<T, ID> {
    pub id: ID,
    pub label: String,
    pub changed: bool,
    pub deleted: bool,
    _f: PhantomData<T>,
}

impl<T: CommonEntity<ID>, ID> EntityInfo<T, ID> {
    pub fn new(label: &str, entity: &T) -> EntityInfo<T, ID> {
        Self {
            id: entity.id(),
            label: label.to_string(),
            changed: entity.changed(),
            deleted: entity.deleted(),
            _f: Default::default(),
        }
    }
}

impl<ID: Ord, T> Eq for EntityInfo<T, ID> {}

impl<ID: Ord + PartialOrd + Eq, T> PartialEq<Self> for EntityInfo<T, ID> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<ID: Ord, T> PartialOrd<Self> for EntityInfo<T, ID> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl<ID: Ord, T> Ord for EntityInfo<T, ID> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

pub struct EntityCatalog<Entity, EntityId: Hash + Eq>
where
    EntityInfo<Entity, EntityId>: for<'a> From<&'a Entity> + Ord,
{
    pub filter: String,
    pub history: Vec<String>,
    pub catalog: Vec<EntityInfo<Entity, EntityId>>,
    filter_fn: Box<dyn Fn(&Entity, &str) -> bool>,
}

impl<Entity, EntityId: Hash + Eq> EntityCatalog<Entity, EntityId>
where
    EntityInfo<Entity, EntityId>: for<'a> From<&'a Entity> + Ord,
{
    pub fn len(&self) -> usize {
        self.catalog.len()
    }
}

impl<Entity, EntityId: Hash + Eq + Copy + Clone> EntityCatalog<Entity, EntityId>
where
    EntityInfo<Entity, EntityId>: for<'a> From<&'a Entity> + Ord,
    Entity: CommonEntity<EntityId> + Clone,
{
    pub fn filter<Map: HolderMapOps<EntityId, Entity>>(&mut self, map: &Map, mode: FilterMode) {
        let r = self.filter.to_lowercase();
        let res: Vec<EntityInfo<Entity, EntityId>> = map
            .values()
            .filter(|v| match mode {
                FilterMode::All => { true }
                FilterMode::Changed => { v.changed() }
                FilterMode::Deleted => { v.deleted() }
            } && (self.filter_fn)(*v, &r))
            .map(|v| v.into())
            .collect();

        let mut ind = None;
        for (i, v) in self.history.iter().enumerate() {
            if v.to_lowercase() == r {
                ind = Some(i);

                break;
            }
        }

        if !r.is_empty() {
            if let Some(i) = ind {
                self.history.remove(i);
            }

            self.history.push(r);
        }

        self.catalog = res;
        self.catalog.sort();
    }
}

/*
----------------------------------------------------------------------------------------------------
----------------------------------------------------------------------------------------------------
*/

pub struct EntityCatalogsHolder {
    pub filter_mode: FilterMode,

    pub npc: EntityCatalog<Npc, NpcId>,
    pub quest: EntityCatalog<Quest, QuestId>,
    pub skill: EntityCatalog<Skill, SkillId>,
    pub weapon: EntityCatalog<Weapon, ItemId>,
    pub armor: EntityCatalog<Armor, ItemId>,
    pub etc_item: EntityCatalog<EtcItem, ItemId>,
    pub item_set: EntityCatalog<ItemSet, ItemSetId>,
    pub recipe: EntityCatalog<Recipe, RecipeId>,
    pub hunting_zone: EntityCatalog<HuntingZone, HuntingZoneId>,
    pub region: EntityCatalog<Region, RegionId>,
    pub raid_info: EntityCatalog<RaidInfo, RaidInfoId>,
    pub daily_mission: EntityCatalog<DailyMission, DailyMissionId>,
    pub animation_combo: EntityCatalog<AnimationCombo, AnimationComboId>,
    pub residence: EntityCatalog<Residence, ResidenceId>,
}

impl EntityCatalogsHolder {
    pub fn new() -> Self {
        Self {
            filter_mode: FilterMode::All,

            npc: EntityCatalog {
                filter: "".to_string(),
                history: vec![],
                catalog: vec![],
                filter_fn: Box::new(|v, s| {
                    if s.is_empty() {
                        true
                    } else if let Some(val) = s.strip_prefix("mesh:") {
                        v.mesh_params.inner.mesh.to_lowercase().contains(val)
                    } else if let Some(val) = s.strip_prefix("texture:") {
                        v.mesh_params
                            .inner
                            .textures
                            .iter()
                            .any(|v| v.to_lowercase().contains(val))
                    } else if let Some(range) = s.strip_prefix("r:") {
                        is_in_range(range, v.id.0)
                    } else if let Ok(id) = u32::from_str(s) {
                        v.id == NpcId(id)
                    } else {
                        v.name.to_lowercase().contains(s)
                    }
                }),
            },
            quest: EntityCatalog {
                filter: "".to_string(),
                history: vec![],
                catalog: vec![],
                filter_fn: Box::new(|v, s| {
                    if s.is_empty() {
                        true
                    } else if let Some(range) = s.strip_prefix("r:") {
                        is_in_range(range, v.id.0)
                    } else if let Ok(id) = u32::from_str(s) {
                        v.id == QuestId(id)
                    } else {
                        v.title.to_lowercase().contains(s)
                    }
                }),
            },
            skill: EntityCatalog {
                filter: "".to_string(),
                history: vec![],
                catalog: vec![],
                filter_fn: Box::new(|v, s| {
                    if s.is_empty() {
                        true
                    } else if let Some(range) = s.strip_prefix("r:") {
                        is_in_range(range, v.id.0)
                    } else if let Some(val) = s.strip_prefix("effect:") {
                        v.visual_effect.to_lowercase().contains(val)
                    } else if let Ok(id) = u32::from_str(s) {
                        v.id == SkillId(id)
                    } else {
                        v.name.to_lowercase().contains(s)
                    }
                }),
            },
            weapon: EntityCatalog {
                filter: "".to_string(),
                history: vec![],
                catalog: vec![],
                filter_fn: Box::new(|v, s| {
                    if s.is_empty() {
                        true
                    } else if let Some(val) = s.strip_prefix("mesh:") {
                        v.mesh_info
                            .iter()
                            .any(|v| v.mesh.to_lowercase().contains(val))
                    } else if let Some(val) = s.strip_prefix("texture:") {
                        v.mesh_info
                            .iter()
                            .flat_map(|v| &v.texture)
                            .any(|v| v.to_lowercase().contains(val))
                    } else if let Some(range) = s.strip_prefix("r:") {
                        is_in_range(range, v.base_info.id.0)
                    } else if let Ok(id) = u32::from_str(s) {
                        v.base_info.id == ItemId(id)
                    } else {
                        v.base_info.name.to_lowercase().contains(s)
                    }
                }),
            },
            armor: EntityCatalog {
                filter: "".to_string(),
                history: vec![],
                catalog: vec![],
                filter_fn: Box::new(|v, s| {
                    if s.is_empty() {
                        true
                    } else if let Some(range) = s.strip_prefix("r:") {
                        is_in_range(range, v.base_info.id.0)
                    } else if let Ok(id) = u32::from_str(s) {
                        v.base_info.id == ItemId(id)
                    } else {
                        v.base_info.name.to_lowercase().contains(s)
                    }
                }),
            },
            etc_item: EntityCatalog {
                filter: "".to_string(),
                history: vec![],
                catalog: vec![],
                filter_fn: Box::new(|v, s| {
                    if s.is_empty() {
                        true
                    } else if let Some(range) = s.strip_prefix("r:") {
                        is_in_range(range, v.base_info.id.0)
                    } else if let Ok(id) = u32::from_str(s) {
                        v.base_info.id == ItemId(id)
                    } else {
                        v.base_info.name.to_lowercase().contains(s)
                    }
                }),
            },
            item_set: EntityCatalog {
                filter: "".to_string(),
                history: vec![],
                catalog: vec![],
                filter_fn: Box::new(|v, s| {
                    if s.is_empty() {
                        true
                    } else if let Some(range) = s.strip_prefix("r:") {
                        is_in_range(range, v.id.0)
                    } else if let Ok(id) = u32::from_str(s) {
                        v.id == ItemSetId(id)
                    } else {
                        false
                    }
                }),
            },
            recipe: EntityCatalog {
                filter: "".to_string(),
                history: vec![],
                catalog: vec![],
                filter_fn: Box::new(|v, s| {
                    if s.is_empty() {
                        true
                    } else if let Some(range) = s.strip_prefix("r:") {
                        is_in_range(range, v.id.0)
                    } else if let Ok(id) = u32::from_str(s) {
                        v.id == RecipeId(id)
                    } else {
                        v.name.to_lowercase().contains(s)
                    }
                }),
            },
            hunting_zone: EntityCatalog {
                filter: "".to_string(),
                history: vec![],
                catalog: vec![],
                filter_fn: Box::new(|v, s| {
                    if s.is_empty() {
                        true
                    } else if let Some(range) = s.strip_prefix("r:") {
                        is_in_range(range, v.id.0)
                    } else if let Ok(id) = u32::from_str(s) {
                        v.id == HuntingZoneId(id)
                    } else {
                        v.name.to_lowercase().contains(s)
                    }
                }),
            },
            region: EntityCatalog {
                filter: "".to_string(),
                history: vec![],
                catalog: vec![],
                filter_fn: Box::new(|v, s| {
                    if s.is_empty() {
                        true
                    } else if let Some(range) = s.strip_prefix("r:") {
                        is_in_range(range, v.id.0)
                    } else if let Ok(id) = u32::from_str(s) {
                        v.id == RegionId(id)
                    } else {
                        v.name.to_lowercase().contains(s)
                    }
                }),
            },
            raid_info: EntityCatalog {
                filter: "".to_string(),
                history: vec![],
                catalog: vec![],
                filter_fn: Box::new(|v, s| {
                    if s.is_empty() {
                        true
                    } else if let Some(range) = s.strip_prefix("r:") {
                        is_in_range(range, v.id.0)
                    } else if let Some(id) = s.strip_prefix("rb:") {
                        if let Ok(id) = u32::from_str(id) {
                            v.raid_id.0 == id
                        } else {
                            false
                        }
                    } else if let Ok(id) = u32::from_str(s) {
                        v.id.0 == id
                    } else {
                        v.desc.to_lowercase().contains(s)
                    }
                }),
            },
            daily_mission: EntityCatalog {
                filter: "".to_string(),
                history: vec![],
                catalog: vec![],
                filter_fn: Box::new(|v, s| {
                    if s.is_empty() {
                        true
                    } else if let Some(range) = s.strip_prefix("r:") {
                        is_in_range(range, v.id.0)
                    } else if let Ok(id) = u32::from_str(s) {
                        v.id.0 == id
                    } else {
                        v.name.to_lowercase().contains(s)
                    }
                }),
            },
            animation_combo: EntityCatalog {
                filter: "".to_string(),
                history: vec![],
                catalog: vec![],
                filter_fn: Box::new(|v, s| {
                    if s.is_empty() {
                        true
                    } else {
                        v.name.to_lowercase().contains(s)
                            || v.anim_0.to_lowercase().contains(s)
                            || v.anim_1.to_lowercase().contains(s)
                            || v.anim_2.to_lowercase().contains(s)
                    }
                }),
            },
            residence: EntityCatalog {
                filter: "".to_string(),
                history: vec![],
                catalog: vec![],
                filter_fn: Box::new(|v, s| {
                    if s.is_empty() {
                        true
                    } else if let Some(range) = s.strip_prefix("r:") {
                        is_in_range(range, v.id.0)
                    } else {
                        v.name.to_lowercase().contains(s)
                    }
                }),
            },
        }
    }
}

pub trait EntityCatalogsOps {
    fn is_empty(&self) -> bool;
}

impl<Entity, EntityId: Hash + Eq> EntityCatalogsOps for EntityCatalog<Entity, EntityId>
where
    EntityInfo<Entity, EntityId>: for<'a> From<&'a Entity> + Ord,
{
    fn is_empty(&self) -> bool {
        self.catalog.is_empty()
    }
}

impl Index<Entity> for EntityCatalogsHolder {
    type Output = dyn EntityCatalogsOps;

    fn index(&self, index: Entity) -> &Self::Output {
        match index {
            Entity::Npc => &self.npc,
            Entity::Quest => &self.quest,
            Entity::Skill => &self.skill,
            Entity::Weapon => &self.weapon,
            Entity::Armor => &self.armor,
            Entity::EtcItem => &self.etc_item,
            Entity::ItemSet => &self.item_set,
            Entity::Recipe => &self.recipe,
            Entity::HuntingZone => &self.hunting_zone,
            Entity::Region => &self.region,
            Entity::RaidInfo => &self.raid_info,
            Entity::DailyMission => &self.daily_mission,
            Entity::AnimationCombo => &self.animation_combo,
            Entity::Residence => &self.residence,
        }
    }
}
