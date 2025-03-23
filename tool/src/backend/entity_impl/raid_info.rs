use crate::backend::editor::entity::{CommonEditorOps, EntityEditParams};
use crate::backend::editor::{CurrentEntity, EditParamsCommonOps, Editors, WindowParams};
use crate::backend::entity_catalog::EntityInfo;
use crate::backend::holder::{FHashMap, HolderMapOps};
use crate::backend::{Backend, HandleAction};
use crate::common::RaidInfoId;
use crate::entity::CommonEntity;
use crate::entity::raid_info::RaidInfo;

pub type RaidInfoEditor = EntityEditParams<RaidInfo, RaidInfoId, (), ()>;

impl HandleAction for WindowParams<RaidInfo, RaidInfoId, (), ()> {
    fn handle_action(&mut self) {}
}

impl Editors {
    pub fn get_opened_raid_info_info(&self) -> Vec<(String, RaidInfoId, bool)> {
        self.raid_info.get_opened_info()
    }

    pub fn open_raid_info(&mut self, id: RaidInfoId, holder: &mut FHashMap<RaidInfoId, RaidInfo>) {
        for (i, q) in self.raid_info.opened.iter().enumerate() {
            if q.inner.initial_id == id {
                self.current_entity = CurrentEntity::RaidInfo(i);

                return;
            }
        }

        if let Some(q) = holder.get(&id) {
            self.current_entity =
                CurrentEntity::RaidInfo(self.raid_info.add(q.clone(), q.id(), false));
        }
    }

    pub fn set_current_raid_info(&mut self, index: usize) {
        if index < self.raid_info.opened.len() {
            self.current_entity = CurrentEntity::RaidInfo(index);
        }
    }

    pub fn create_new_raid_info(&mut self) {
        self.current_entity = CurrentEntity::RaidInfo(self.raid_info.add_new());
    }
}

impl Backend {
    pub fn filter_raid_info(&mut self) {
        self.entity_catalogs.raid_info.filter(
            &self.holders.game_data_holder.raid_info_holder,
            self.entity_catalogs.filter_mode,
        );
    }

    pub fn save_raid_info_from_dlg(&mut self, id: RaidInfoId) {
        if let CurrentEntity::RaidInfo(index) = self.editors.current_entity {
            let new_entity = self.editors.raid_info.opened.get_mut(index).unwrap();

            if new_entity.inner.inner.id() != id {
                return;
            }

            new_entity.inner.initial_id = new_entity.inner.inner.id;

            let entity = new_entity.inner.inner.clone();

            new_entity.on_save();

            self.save_raid_info_object_force(entity);
        }
    }

    pub(crate) fn save_raid_info_object_force(&mut self, mut v: RaidInfo) {
        if let Some(vv) = self.holders.game_data_holder.raid_info_holder.get(&v.id) {
            if *vv == v {
                return;
            }
        }
        v._changed = true;

        self.holders
            .game_data_holder
            .raid_info_holder
            .insert(v.id, v);

        self.filter_raid_info();
        self.check_for_unwrote_changed();
    }
}

impl From<&RaidInfo> for EntityInfo<RaidInfo, RaidInfoId> {
    fn from(value: &RaidInfo) -> Self {
        EntityInfo::new(
            &format!("ID: {}\nNpcId: {}", value.id.0, value.raid_id.0),
            value,
        )
    }
}
