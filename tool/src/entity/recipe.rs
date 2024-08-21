use crate::data::{ItemId, RecipeId};
use crate::entity::{CommonEntity, GetEditParams};
use serde::{Deserialize, Serialize};

impl GetEditParams<()> for Recipe {
    fn edit_params(&self) {}
}

impl CommonEntity<RecipeId> for Recipe {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn desc(&self) -> String {
        "".to_string()
    }

    fn id(&self) -> RecipeId {
        self.id
    }

    fn changed(&self) -> bool {
        self._changed
    }

    fn deleted(&self) -> bool {
        self._deleted
    }

    fn new(id: RecipeId) -> Self {
        Recipe {
            id,
            name: "New Recipe".to_string(),
            recipe_item: ItemId(1),
            level: 1,
            product: ItemId(1),
            product_count: 1,
            show_tree: true,
            is_multiple_product: false,
            mp_consume: 100,
            success_rate: 100,
            materials: vec![RecipeMaterial {
                id: ItemId(2),
                count: 1,
                recipe_id: RecipeId(0),
            }],

            _changed: false,
            _deleted: false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct RecipeMaterial {
    pub(crate) id: ItemId,
    pub(crate) count: u32,
    pub(crate) recipe_id: RecipeId,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct Recipe {
    pub(crate) id: RecipeId,
    pub(crate) name: String,
    pub(crate) recipe_item: ItemId,
    pub(crate) level: u32,

    pub(crate) product: ItemId,
    pub(crate) product_count: u32,

    pub(crate) show_tree: bool,
    pub(crate) is_multiple_product: bool,

    pub(crate) mp_consume: u32,
    pub(crate) success_rate: u32,

    pub(crate) materials: Vec<RecipeMaterial>,

    #[serde(skip)]
    pub _changed: bool,
    #[serde(skip)]
    pub _deleted: bool,
}
