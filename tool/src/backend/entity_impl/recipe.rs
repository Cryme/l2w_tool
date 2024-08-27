use crate::backend::entity_catalog::EntityInfo;
use crate::backend::entity_editor::{
    CommonEditorOps, CurrentEntity, EditParams, EditParamsCommonOps, EntityEditParams, WindowParams,
};
use crate::backend::holder::{FHashMap, HolderMapOps};
use crate::backend::{Backend, HandleAction};
use crate::common::RecipeId;
use crate::entity::recipe::Recipe;
use crate::entity::CommonEntity;
use serde::{Deserialize, Serialize};

pub type RecipeEditor = EntityEditParams<Recipe, RecipeId, RecipeAction, ()>;

impl HandleAction for WindowParams<Recipe, RecipeId, RecipeAction, ()> {
    fn handle_action(&mut self) {
        let item = self;

        let mut action = item.action.write().unwrap();

        match *action {
            RecipeAction::DeleteIngredient(i) => {
                item.inner.materials.remove(i);
            }

            RecipeAction::None => {}
        }

        *action = RecipeAction::None;
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum RecipeAction {
    #[default]
    None,
    DeleteIngredient(usize),
}

impl EditParams {
    pub fn get_opened_recipes_info(&self) -> Vec<(String, RecipeId, bool)> {
        self.recipes.get_opened_info()
    }

    pub fn open_recipe(&mut self, id: RecipeId, holder: &mut FHashMap<RecipeId, Recipe>) {
        for (i, q) in self.recipes.opened.iter().enumerate() {
            if q.inner.initial_id == id {
                self.current_entity = CurrentEntity::Recipe(i);

                return;
            }
        }

        if let Some(q) = holder.get(&id) {
            self.current_entity = CurrentEntity::Recipe(self.recipes.add(q.clone(), q.id(), false));
        }
    }

    pub fn set_current_recipe(&mut self, index: usize) {
        if index < self.recipes.opened.len() {
            self.current_entity = CurrentEntity::Recipe(index);
        }
    }

    pub fn create_new_recipe(&mut self) {
        self.current_entity = CurrentEntity::Recipe(self.recipes.add_new());
    }
}

impl Backend {
    pub fn filter_recipes(&mut self) {
        self.entity_catalogs.recipe.filter(
            &self.holders.game_data_holder.recipe_holder,
            self.entity_catalogs.filter_mode,
        );
    }

    pub fn save_recipe_from_dlg(&mut self, id: RecipeId) {
        if let CurrentEntity::Recipe(index) = self.edit_params.current_entity {
            let new_entity = self.edit_params.recipes.opened.get_mut(index).unwrap();

            if new_entity.inner.inner.id() != id {
                return;
            }

            new_entity.inner.initial_id = new_entity.inner.inner.id;

            let entity = new_entity.inner.inner.clone();

            new_entity.on_save();

            self.save_recipe_force(entity);
        }
    }

    pub(crate) fn save_recipe_force(&mut self, mut v: Recipe) {
        if let Some(vv) = self.holders.game_data_holder.recipe_holder.get(&v.id) {
            if *vv == v {
                return;
            }
        }
        v._changed = true;

        self.holders.game_data_holder.recipe_holder.insert(v.id, v);

        self.filter_recipes();
        self.check_for_unwrote_changed();
    }
}

impl From<&Recipe> for EntityInfo<Recipe, RecipeId> {
    fn from(value: &Recipe) -> Self {
        EntityInfo::new(&format!("ID: {}\n{}", value.id.0, value.name), value)
    }
}
