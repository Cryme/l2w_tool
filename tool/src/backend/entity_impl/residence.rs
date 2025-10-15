use crate::backend::editor::entity::{CommonEditorOps, EntityEditParams};
use crate::backend::editor::{CurrentEntity, EditParamsCommonOps, Editors, WindowParams};
use crate::backend::entity_catalog::EntityInfo;
use crate::backend::holder::{FHashMap, HolderMapOps};
use crate::backend::{Backend, HandleAction};
use crate::common::ResidenceId;
use crate::entity::CommonEntity;
use crate::entity::residence::Residence;
use serde::{Deserialize, Serialize};

pub type ResidenceEditor = EntityEditParams<Residence, ResidenceId, ResidenceAction, ()>;

impl HandleAction for WindowParams<Residence, ResidenceId, ResidenceAction, ()> {
    fn handle_action(&mut self) {
        let item = self;

        let mut action = item.action.write().unwrap();

        *action = ResidenceAction::None;
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum ResidenceAction {
    #[default]
    None,
}

impl Editors {
    pub fn get_opened_residences_info(&self) -> Vec<(String, ResidenceId, bool)> {
        self.residences.get_opened_info()
    }

    pub fn open_residence(
        &mut self,
        id: ResidenceId,
        holder: &mut FHashMap<ResidenceId, Residence>,
    ) {
        for (i, q) in self.residences.opened.iter().enumerate() {
            if q.inner.initial_id == id {
                self.current_entity = CurrentEntity::Residence(i);

                return;
            }
        }

        if let Some(q) = holder.get(&id) {
            self.current_entity =
                CurrentEntity::Residence(self.residences.add(q.clone(), q.id(), false));
        }
    }

    pub fn set_current_residence(&mut self, index: usize) {
        if index < self.residences.opened.len() {
            self.current_entity = CurrentEntity::Residence(index);
        }
    }

    pub fn create_new_residence(&mut self) {
        self.current_entity = CurrentEntity::Residence(self.residences.add_new());
    }
}

impl Backend {
    pub fn filter_residences(&mut self) {
        self.entity_catalogs.residence.filter(
            &self.holders.game_data_holder.residence_holder,
            self.entity_catalogs.filter_mode,
        );
    }

    pub fn save_residence_from_dlg(&mut self, id: ResidenceId) {
        if let CurrentEntity::Residence(index) = self.editors.current_entity {
            let new_entity = self.editors.residences.opened.get_mut(index).unwrap();

            if new_entity.inner.inner.id() != id {
                return;
            }

            new_entity.inner.initial_id = new_entity.inner.inner.id;

            let entity = new_entity.inner.inner.clone();

            new_entity.on_save();

            self.save_residence_force(entity);
        }
    }

    pub(crate) fn save_residence_force(&mut self, mut v: Residence) {
        if let Some(vv) = self.holders.game_data_holder.residence_holder.get(&v.id)
            && *vv == v {
                return;
            }
        v._changed = true;

        self.holders
            .game_data_holder
            .residence_holder
            .insert(v.id, v);

        self.filter_residences();
        self.check_for_unwrote_changed();
    }
}

impl From<&Residence> for EntityInfo<Residence, ResidenceId> {
    fn from(value: &Residence) -> Self {
        EntityInfo::new(&format!("ID: {}\n{}", value.id.0, value.name.ru), value)
    }
}
