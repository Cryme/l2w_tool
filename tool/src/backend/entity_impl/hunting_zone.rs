use crate::backend::editor::entity::{CommonEditorOps, EntityEditParams};
use crate::backend::editor::{CurrentEntity, EditParamsCommonOps, Editors, WindowParams};
use crate::backend::entity_catalog::EntityInfo;
use crate::backend::holder::{FHashMap, HolderMapOps};
use crate::backend::{Backend, HandleAction};
use crate::common::HuntingZoneId;
use crate::entity::hunting_zone::HuntingZone;
use crate::entity::CommonEntity;
use serde::{Deserialize, Serialize};

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

impl Editors {
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
                CurrentEntity::HuntingZone(self.hunting_zones.add(q.clone(), q.id(), false));
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
        self.entity_catalogs.hunting_zone.filter(
            &self.holders.game_data_holder.hunting_zone_holder,
            self.entity_catalogs.filter_mode,
        );
    }

    pub fn save_hunting_zone_from_dlg(&mut self, id: HuntingZoneId) {
        if let CurrentEntity::HuntingZone(index) = self.editors.current_entity {
            let new_entity = self.editors.hunting_zones.opened.get_mut(index).unwrap();

            if new_entity.inner.inner.id() != id {
                return;
            }

            new_entity.inner.initial_id = new_entity.inner.inner.id;

            let entity = new_entity.inner.inner.clone();

            new_entity.on_save();

            self.save_hunting_zone_object_force(entity);
        }
    }

    pub(crate) fn save_hunting_zone_object_force(&mut self, mut v: HuntingZone) {
        if let Some(vv) = self.holders.game_data_holder.hunting_zone_holder.get(&v.id) {
            if *vv == v {
                return;
            }
        }
        v._changed = true;

        self.holders
            .game_data_holder
            .hunting_zone_holder
            .insert(v.id, v);

        self.filter_hunting_zones();
        self.check_for_unwrote_changed();
    }
}

impl From<&HuntingZone> for EntityInfo<HuntingZone, HuntingZoneId> {
    fn from(value: &HuntingZone) -> Self {
        EntityInfo::new(&format!("ID: {}\n{}", value.id.0, value.name), value)
    }
}
