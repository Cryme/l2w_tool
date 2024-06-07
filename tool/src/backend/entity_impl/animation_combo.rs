use crate::backend::entity_catalog::EntityInfo;
use crate::backend::entity_editor::{
    CommonEditorOps, CurrentEntity, EditParams, EditParamsCommonOps, EntityEditParams, WindowParams,
};
use crate::backend::holder::{FDHashMap, HolderMapOps};
use crate::backend::{Backend, HandleAction};
use crate::data::AnimationComboId;
use crate::entity::animation_combo::AnimationCombo;
use crate::entity::CommonEntity;

pub type AnimationComboEditor = EntityEditParams<AnimationCombo, AnimationComboId, (), ()>;

impl HandleAction for WindowParams<AnimationCombo, AnimationComboId, (), ()> {
    fn handle_action(&mut self) {}
}

impl EditParams {
    pub fn get_opened_animation_combo_info(&self) -> Vec<(String, AnimationComboId, bool)> {
        self.animation_combo.get_opened_info()
    }

    pub fn open_animation_combo(
        &mut self,
        id: AnimationComboId,
        holder: &mut FDHashMap<AnimationComboId, AnimationCombo>,
    ) {
        for (i, q) in self.animation_combo.opened.iter().enumerate() {
            if q.inner.initial_id == id {
                self.current_entity = CurrentEntity::AnimationCombo(i);

                return;
            }
        }

        if let Some(q) = holder.get(&id) {
            self.current_entity =
                CurrentEntity::AnimationCombo(self.animation_combo.add(q.clone(), q.id(), false));
        }
    }

    pub fn set_current_animation_combo(&mut self, index: usize) {
        if index < self.animation_combo.opened.len() {
            self.current_entity = CurrentEntity::AnimationCombo(index);
        }
    }

    pub fn create_new_animation_combo(&mut self) {
        self.current_entity = CurrentEntity::AnimationCombo(self.animation_combo.add_new());
    }
}

impl Backend {
    pub fn filter_animation_combo(&mut self) {
        self.entity_catalogs.animation_combo.filter(
            &self.holders.game_data_holder.animation_combo_holder,
            self.entity_catalogs.filter_mode,
        );
    }

    pub fn save_animation_combo_from_dlg(&mut self, id: AnimationComboId) {
        if let CurrentEntity::AnimationCombo(index) = self.edit_params.current_entity {
            let new_entity = self
                .edit_params
                .animation_combo
                .opened
                .get_mut(index)
                .unwrap();

            if new_entity.inner.inner.id() != id {
                return;
            }

            new_entity.inner.initial_id = new_entity.inner.inner.id;

            let entity = new_entity.inner.inner.clone();

            new_entity.on_save();

            self.save_animation_combo_object_force(entity);
        }
    }

    pub(crate) fn save_animation_combo_object_force(&mut self, mut v: AnimationCombo) {
        if let Some(vv) = self
            .holders
            .game_data_holder
            .animation_combo_holder
            .get(&v.id)
        {
            if *vv == v {
                return;
            }
        }
        v._changed = true;

        self.holders
            .game_data_holder
            .animation_combo_holder
            .insert(v.id, v);

        self.filter_animation_combo();
        self.check_for_unwrote_changed();
    }
}

impl From<&AnimationCombo> for EntityInfo<AnimationCombo, AnimationComboId> {
    fn from(value: &AnimationCombo) -> Self {
        EntityInfo::new(&format!("Name: {}\n", value.name), value)
    }
}
