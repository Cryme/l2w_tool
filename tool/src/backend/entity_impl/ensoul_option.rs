use crate::backend::editor::entity::{CommonEditorOps, EntityEditParams};
use crate::backend::editor::{CurrentEntity, EditParamsCommonOps, Editors, WindowParams};
use crate::backend::entity_catalog::EntityInfo;
use crate::backend::holder::{FHashMap, HolderMapOps};
use crate::backend::{Backend, HandleAction};
use crate::common::EnsoulOptionId;
use crate::entity::ensoul_option::EnsoulOption;
use crate::entity::{CommonEntity, GameEntityT};
use serde::{Deserialize, Serialize};

pub type EnsoulOptionEditor =
    EntityEditParams<EnsoulOption, EnsoulOptionId, EnsoulOptionAction, ()>;

impl HandleAction for WindowParams<EnsoulOption, EnsoulOptionId, EnsoulOptionAction, ()> {
    fn handle_action(&mut self) {
        let item = self;

        let mut action = item.action.write().unwrap();

        match *action {
            EnsoulOptionAction::None => {}
        }

        *action = EnsoulOptionAction::None;
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum EnsoulOptionAction {
    #[default]
    None,
}

impl Editors {
    pub fn get_opened_ensoul_option_info(&self) -> Vec<(String, EnsoulOptionId, bool)> {
        self.ensoul_options.get_opened_info()
    }

    pub fn force_update_ensoul_option(&mut self, item: &EnsoulOption) {
        if item._deleted {
            self.close_if_opened(GameEntityT::EnsoulOption(item.id));
        } else if let Some(v) = self
            .ensoul_options
            .opened
            .iter_mut()
            .find(|v| v.inner.inner.id() == item.id())
        {
            v.inner.inner = item.clone();
        }
    }

    pub fn open_ensoul_option(
        &mut self,
        id: EnsoulOptionId,
        holder: &mut FHashMap<EnsoulOptionId, EnsoulOption>,
    ) {
        for (i, q) in self.ensoul_options.opened.iter().enumerate() {
            if q.inner.initial_id == id {
                self.current_entity = CurrentEntity::EnsoulOption(i);

                return;
            }
        }

        if let Some(q) = holder.get(&id) {
            self.current_entity =
                CurrentEntity::EnsoulOption(self.ensoul_options.add(q.clone(), q.id(), false));
        }
    }

    pub fn set_current_ensoul_option(&mut self, index: usize) {
        if index < self.ensoul_options.opened.len() {
            self.current_entity = CurrentEntity::EnsoulOption(index);
        }
    }

    pub fn create_new_ensoul_option(&mut self) {
        self.current_entity = CurrentEntity::EnsoulOption(self.ensoul_options.add_new());
    }
}

impl Backend {
    pub fn filter_ensoul_option(&mut self) {
        self.entity_catalogs.ensoul_option.filter(
            &self.holders.game_data_holder.ensoul_option_holder,
            self.entity_catalogs.filter_mode,
        );
    }

    pub fn save_ensoul_option_from_dlg(&mut self, id: EnsoulOptionId) {
        if let CurrentEntity::EnsoulOption(index) = self.editors.current_entity {
            let new_entity = self.editors.ensoul_options.opened.get_mut(index).unwrap();

            if new_entity.inner.inner.id() != id {
                return;
            }

            new_entity.inner.initial_id = new_entity.inner.inner.id;

            let entity = new_entity.inner.inner.clone();

            new_entity.on_save();

            self.save_ensoul_option_force(entity);
        }
    }

    pub fn save_ensoul_option_force(&mut self, mut v: EnsoulOption) {
        if let Some(vv) = self
            .holders
            .game_data_holder
            .ensoul_option_holder
            .get(&v.id)
        {
            if *vv == v {
                return;
            }
        }

        v._changed = true;

        self.holders
            .game_data_holder
            .ensoul_option_holder
            .insert(v.id, v);

        self.filter_ensoul_option();
        self.check_for_unwrote_changed();
    }
}

impl From<&EnsoulOption> for EntityInfo<EnsoulOption, EnsoulOptionId> {
    fn from(value: &EnsoulOption) -> Self {
        EntityInfo::new(
            &format!(
                "ID: {}\n{} {} {}",
                value.id.0, value.name, value.option_type, value.step
            ),
            value,
        )
    }
}
