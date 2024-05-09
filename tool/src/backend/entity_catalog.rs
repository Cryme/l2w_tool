use std::hash::Hash;
use std::marker::PhantomData;
use std::str::FromStr;
use crate::backend::entity_impl::hunting_zone::HuntingZoneInfo;
use crate::backend::entity_impl::item::armor::ArmorInfo;
use crate::backend::entity_impl::item::etc_item::EtcItemInfo;
use crate::backend::entity_impl::item::weapon::WeaponInfo;
use crate::backend::entity_impl::item_set::ItemSetInfo;
use crate::backend::entity_impl::npc::NpcInfo;
use crate::backend::entity_impl::quest::QuestInfo;
use crate::backend::entity_impl::recipe::RecipeInfo;
use crate::backend::entity_impl::region::RegionInfo;
use crate::backend::entity_impl::skill::SkillInfo;
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

pub struct EntityCatalog<Entity, EntityId: Hash+Eq, EntityInfo: for<'a> From<&'a Entity>+Ord> {
    pub filter: String,
    pub history: Vec<String>,
    pub catalog: Vec<EntityInfo>,
    filter_fn: Box<dyn Fn(&Entity, &str) -> bool>,
    _f: PhantomData<EntityId>,
}

impl<Entity, EntityId: Hash+Eq, EntityInfo: for<'a> From<&'a Entity>+Ord> EntityCatalog<Entity, EntityId, EntityInfo> {
    pub fn filter(&mut self, map: &FHashMap<EntityId, Entity>) {
        let r = self.filter.to_lowercase();
        let res: Vec<EntityInfo> = map.inner.values().filter(|v| (self.filter_fn)(*v, &r)).map(|v| v.into()).collect();

        let mut ind = None;
        for (i, v) in self.history.iter().enumerate() {
            if v.to_lowercase() == r {
                ind = Some(i);

                break
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
    pub npc: EntityCatalog<Npc, NpcId, NpcInfo>,
    pub quest: EntityCatalog<Quest, QuestId, QuestInfo>,
    pub skill: EntityCatalog<Skill, SkillId, SkillInfo>,
    pub weapon: EntityCatalog<Weapon, ItemId, WeaponInfo>,
    pub armor: EntityCatalog<Armor, ItemId, ArmorInfo>,
    pub etc_item: EntityCatalog<EtcItem, ItemId, EtcItemInfo>,
    pub item_set: EntityCatalog<ItemSet, ItemSetId, ItemSetInfo>,
    pub recipe: EntityCatalog<Recipe, RecipeId, RecipeInfo>,
    pub hunting_zone: EntityCatalog<HuntingZone, HuntingZoneId, HuntingZoneInfo>,
    pub region: EntityCatalog<Region, RegionId, RegionInfo>,
}

impl EntityCatalogsHolder {
    pub fn new() -> Self {
        Self {
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
                        v.mesh_params.inner.textures.iter().any(|v| v.to_lowercase().contains(&s[8..]))
                    } else if let Ok(id) = u32::from_str(&s) {
                        v.id == NpcId(id)
                    } else {
                        v.name.to_lowercase().contains(&s)
                    }
                }),
                _f: Default::default(),
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
                _f: Default::default(),
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
                _f: Default::default(),
            },
            weapon: EntityCatalog {
                filter: "".to_string(),
                history: vec![],
                catalog: vec![],
                filter_fn: Box::new(|v, s| {
                    if s.is_empty() {
                        true
                    } else if s.starts_with("mesh:") {
                        v.mesh_info.iter().any(|v| v.texture.to_lowercase().contains(&s[5..]))
                    } else if s.starts_with("texture:") {
                        v.mesh_info.iter().any(|v| v.mesh.to_lowercase().contains(&s[8..]))
                    } else if let Ok(id) = u32::from_str(&s) {
                        v.base_info.id == ItemId(id)
                    } else {
                        v.base_info.name.to_lowercase().contains(&s)
                    }
                }),
                _f: Default::default(),
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
                _f: Default::default(),
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
                _f: Default::default(),
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
                _f: Default::default(),
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
                _f: Default::default(),
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
                _f: Default::default(),
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
                _f: Default::default(),
            },
        }
    }
}
