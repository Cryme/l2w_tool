use crate::backend::holder::FHashMap;
use crate::backend::{
    Backend, HandleAction,
};
use crate::data::RecipeId;
use crate::entity::recipe::Recipe;
use crate::entity::CommonEntity;
use serde::{Deserialize, Serialize};
use crate::backend::entity_editor::{CommonEditorOps, CurrentEntity, EditParams, EntityEditParams, WindowParams};

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
            self.current_entity = CurrentEntity::Recipe(self.recipes.add(q.clone(), q.id()));
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
        self.entity_catalogs.recipe.filter(&self.holders.game_data_holder.recipe_holder);
    }

    pub fn save_recipe_from_dlg(&mut self, id: RecipeId) {
        if let CurrentEntity::Recipe(index) = self.edit_params.current_entity {
            let new_entity = self.edit_params.recipes.opened.get_mut(index).unwrap();

            if new_entity.inner.inner.id() != id {
                return;
            }

            new_entity.inner.initial_id = new_entity.inner.inner.id;

            let entity = new_entity.inner.inner.clone();

            self.save_recipe_force(entity);
        }
    }

    pub(crate) fn save_recipe_force(&mut self, v: Recipe) {
        if let Some(vv) = self.holders.game_data_holder.recipe_holder.get(&v.id) {
            if *vv == v{
                return;
            }
        }
        self.set_changed();

        self.holders.game_data_holder.recipe_holder.insert(v.id, v);

        self.filter_recipes();
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub struct RecipeInfo {
    pub(crate) id: RecipeId,
    pub(crate) name: String,
}

impl From<&Recipe> for RecipeInfo {
    fn from(value: &Recipe) -> Self {
        RecipeInfo {
            id: value.id,
            name: value.name.clone(),
        }
    }
}
