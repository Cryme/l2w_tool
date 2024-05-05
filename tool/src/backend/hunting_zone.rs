use crate::backend::{
    Backend, CommonEditorOps, CurrentEntity, EditParams, EntityEditParams, HandleAction,
    WindowParams,
};
use crate::data::HuntingZoneId;
use crate::entity::hunting_zone::HuntingZone;
use crate::entity::CommonEntity;
use crate::holder::FHashMap;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub type HuntingZoneEditor = EntityEditParams<HuntingZone, HuntingZoneId, HuntingZoneAction, ()>;

impl HandleAction for WindowParams<HuntingZone, HuntingZoneId, HuntingZoneAction, ()> {
    fn handle_action(&mut self) {
        let item = self;

        let mut action = item.action.write().unwrap();

        match *action {
            HuntingZoneAction::RemoveQuest(i) => {
                item.inner.quests.remove(i);
            }

            HuntingZoneAction::RemoveMapObject(i) => {
                item.inner.world_map_objects.remove(i);
            }

            HuntingZoneAction::None => {}
        }

        *action = HuntingZoneAction::None;

        for v in &mut item.inner.world_map_objects {
            let mut action = v.action.write().unwrap();
            match *action {
                MapObjectAction::RemoveUnk1(i) => {
                    v.inner.unk1.remove(i);
                }

                MapObjectAction::None => {}
            }

            *action = MapObjectAction::None;
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum HuntingZoneAction {
    #[default]
    None,
    RemoveQuest(usize),
    RemoveMapObject(usize),
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum MapObjectAction {
    #[default]
    None,
    RemoveUnk1(usize),
}

impl EditParams {
    pub fn get_opened_hunting_zone_info(&self) -> Vec<(String, HuntingZoneId, bool)> {
        self.hunting_zones.get_opened_info()
    }

    pub fn open_hunting_zone(
        &mut self,
        id: HuntingZoneId,
        holder: &mut FHashMap<HuntingZoneId, HuntingZone>,
    ) {
        for (i, q) in self.hunting_zones.opened.iter().enumerate() {
            if q.inner.initial_id == id {
                self.current_entity = CurrentEntity::HuntingZone(i);

                return;
            }
        }

        if let Some(q) = holder.get(&id) {
            self.current_entity =
                CurrentEntity::HuntingZone(self.hunting_zones.add(q.clone(), q.id()));
        }
    }

    pub fn set_current_hunting_zone(&mut self, index: usize) {
        if index < self.hunting_zones.opened.len() {
            self.current_entity = CurrentEntity::HuntingZone(index);
        }
    }

    pub fn create_new_hunting_zone(&mut self) {
        self.current_entity = CurrentEntity::HuntingZone(self.hunting_zones.add_new());
    }
}

impl Backend {
    pub fn filter_hunting_zones(&mut self) {
        let s = self.filter_params.hunting_zone_filter_string.to_lowercase();

        let fun: Box<dyn Fn(&&HuntingZone) -> bool> = if s.is_empty() {
            Box::new(|_: &&HuntingZone| true)
        } else if let Ok(id) = u32::from_str(&s) {
            Box::new(move |v: &&HuntingZone| v.id.0 == id)
        } else {
            Box::new(|v: &&HuntingZone| {
                v.name.to_lowercase().contains(&s) || v.desc.to_lowercase().contains(&s)
            })
        };

        self.filter_params.hunting_zone_catalog = self
            .holders
            .game_data_holder
            .hunting_zone_holder
            .values()
            .filter(fun)
            .map(HuntingZoneInfo::from)
            .collect();

        self.filter_params
            .hunting_zone_catalog
            .sort_by(|a, b| a.id.cmp(&b.id))
    }

    pub fn save_hunting_zone_from_dlg(&mut self, id: HuntingZoneId) {
        if let CurrentEntity::HuntingZone(index) = self.edit_params.current_entity {
            let new_entity = self.edit_params.hunting_zones.opened.get(index).unwrap();

            if new_entity.inner.inner.id() != id {
                return;
            }

            self.save_hunting_zone_object_force(new_entity.inner.inner.clone());
        }
    }

    pub(crate) fn save_hunting_zone_object_force(&mut self, v: HuntingZone) {
        self.holders
            .game_data_holder
            .hunting_zone_holder
            .insert(v.id, v);

        self.filter_hunting_zones();
    }
}

pub struct HuntingZoneInfo {
    pub(crate) id: HuntingZoneId,
    pub(crate) name: String,
}

impl From<&HuntingZone> for HuntingZoneInfo {
    fn from(value: &HuntingZone) -> Self {
        HuntingZoneInfo {
            id: value.id,
            name: value.name.clone(),
        }
    }
}
