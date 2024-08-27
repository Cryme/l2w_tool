use crate::backend::entity_catalog::EntityInfo;
use crate::backend::entity_editor::{
    CommonEditorOps, CurrentEntity, EditParams, EditParamsCommonOps, EntityEditParams, WindowParams,
};
use crate::backend::holder::{FHashMap, HolderMapOps};
use crate::backend::{Backend, HandleAction};
use crate::common::DailyMissionId;
use crate::entity::daily_mission::DailyMission;
use crate::entity::CommonEntity;
use serde::{Deserialize, Serialize};

pub type DailyMissionEditor =
    EntityEditParams<DailyMission, DailyMissionId, DailyMissionAction, ()>;

impl HandleAction for WindowParams<DailyMission, DailyMissionId, DailyMissionAction, ()> {
    fn handle_action(&mut self) {
        let mut action = self.action.write().unwrap();

        match *action {
            DailyMissionAction::RemoveUnk7(i) => {
                self.inner.unk7.remove(i);
            }
            DailyMissionAction::RemoveReward(i) => {
                self.inner.rewards.remove(i);
            }

            DailyMissionAction::None => {}
        }

        *action = DailyMissionAction::None;
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum DailyMissionAction {
    #[default]
    None,
    RemoveUnk7(usize),
    RemoveReward(usize),
}

impl EditParams {
    pub fn get_opened_daily_missions_info(&self) -> Vec<(String, DailyMissionId, bool)> {
        self.daily_mission.get_opened_info()
    }

    pub fn open_daily_mission(
        &mut self,
        id: DailyMissionId,
        holder: &mut FHashMap<DailyMissionId, DailyMission>,
    ) {
        for (i, q) in self.daily_mission.opened.iter().enumerate() {
            if q.inner.initial_id == id {
                self.current_entity = CurrentEntity::DailyMission(i);

                return;
            }
        }

        if let Some(q) = holder.get(&id) {
            self.current_entity =
                CurrentEntity::DailyMission(self.daily_mission.add(q.clone(), q.id(), false));
        }
    }

    pub fn set_current_daily_mission(&mut self, index: usize) {
        if index < self.daily_mission.opened.len() {
            self.current_entity = CurrentEntity::DailyMission(index);
        }
    }

    pub fn create_new_daily_mission(&mut self) {
        self.current_entity = CurrentEntity::DailyMission(self.daily_mission.add_new());
    }
}

impl Backend {
    pub fn filter_daily_mission(&mut self) {
        self.entity_catalogs.daily_mission.filter(
            &self.holders.game_data_holder.daily_mission_holder,
            self.entity_catalogs.filter_mode,
        );
    }

    pub fn save_daily_mission_from_dlg(&mut self, id: DailyMissionId) {
        if let CurrentEntity::DailyMission(index) = self.edit_params.current_entity {
            let new_entity = self
                .edit_params
                .daily_mission
                .opened
                .get_mut(index)
                .unwrap();

            if new_entity.inner.inner.id() != id {
                return;
            }

            new_entity.inner.initial_id = new_entity.inner.inner.id;

            let entity = new_entity.inner.inner.clone();

            new_entity.on_save();

            self.save_daily_mission_object_force(entity);
        }
    }

    pub(crate) fn save_daily_mission_object_force(&mut self, mut v: DailyMission) {
        if let Some(vv) = self
            .holders
            .game_data_holder
            .daily_mission_holder
            .get(&v.id)
        {
            if *vv == v {
                return;
            }
        }
        v._changed = true;

        self.holders
            .game_data_holder
            .daily_mission_holder
            .insert(v.id, v);

        self.filter_daily_mission();
        self.check_for_unwrote_changed();
    }
}

impl From<&DailyMission> for EntityInfo<DailyMission, DailyMissionId> {
    fn from(value: &DailyMission) -> Self {
        EntityInfo::new(&format!("ID: {}\n{}", value.id.0, value.name), value)
    }
}
