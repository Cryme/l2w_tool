use crate::backend::holder::FHashMap;
use crate::data::{HuntingZoneId, ItemId, ItemSetId, NpcId, QuestId, RecipeId, RegionId, SkillId};
use crate::entity::hunting_zone::HuntingZone;
use crate::entity::item::armor::Armor;
use crate::entity::item::etc_item::EtcItem;
use crate::entity::item::weapon::Weapon;
use crate::entity::item_set::ItemSet;
use crate::entity::npc::Npc;
use crate::entity::quest::Quest;
use crate::entity::recipe::Recipe;
use crate::entity::region::Region;
use crate::entity::skill::Skill;
use crate::entity::CommonEntity;
use std::cmp::Ordering;
use std::hash::Hash;
use std::marker::PhantomData;
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
        self.id.partial_cmp(&other.id)
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
    Entity: CommonEntity<EntityId>,
{
    pub fn filter(&mut self, map: &FHashMap<EntityId, Entity>, mode: FilterMode) {
        let r = self.filter.to_lowercase();
        let res: Vec<EntityInfo<Entity, EntityId>> = map
            .inner
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
                    } else if s.starts_with("mesh:") {
                        v.mesh_params.inner.mesh.to_lowercase().contains(&s[5..])
                    } else if s.starts_with("texture:") {
                        v.mesh_params
                            .inner
                            .textures
                            .iter()
                            .any(|v| v.to_lowercase().contains(&s[8..]))
                    } else if let Ok(id) = u32::from_str(&s) {
                        v.id == NpcId(id)
                    } else {
                        v.name.to_lowercase().contains(&s)
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
                    } else if let Ok(id) = u32::from_str(&s) {
                        v.id == QuestId(id)
                    } else {
                        v.title.to_lowercase().contains(&s)
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
                    } else if let Ok(id) = u32::from_str(&s) {
                        v.id == SkillId(id)
                    } else {
                        v.name.to_lowercase().contains(&s)
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
                    } else if s.starts_with("mesh:") {
                        v.mesh_info
                            .iter()
                            .any(|v| v.texture.to_lowercase().contains(&s[5..]))
                    } else if s.starts_with("texture:") {
                        v.mesh_info
                            .iter()
                            .any(|v| v.mesh.to_lowercase().contains(&s[8..]))
                    } else if let Ok(id) = u32::from_str(&s) {
                        v.base_info.id == ItemId(id)
                    } else {
                        v.base_info.name.to_lowercase().contains(&s)
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
                    } else if let Ok(id) = u32::from_str(&s) {
                        v.base_info.id == ItemId(id)
                    } else {
                        v.base_info.name.to_lowercase().contains(&s)
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
                    } else if let Ok(id) = u32::from_str(&s) {
                        v.base_info.id == ItemId(id)
                    } else {
                        v.base_info.name.to_lowercase().contains(&s)
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
                    } else if let Ok(id) = u32::from_str(&s) {
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
                    } else if let Ok(id) = u32::from_str(&s) {
                        v.id == RecipeId(id)
                    } else {
                        v.name.to_lowercase().contains(&s)
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
                    } else if let Ok(id) = u32::from_str(&s) {
                        v.id == HuntingZoneId(id)
                    } else {
                        v.name.to_lowercase().contains(&s)
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
                    } else if let Ok(id) = u32::from_str(&s) {
                        v.id == RegionId(id)
                    } else {
                        v.name.to_lowercase().contains(&s)
                    }
                }),
            },
        }
    }
}
