use crate::entity::recipe::{Recipe, RecipeMaterial};
use crate::holder::grand_crusade_110::{L2GeneralStringTable, Loader110};
use crate::util::l2_reader::{deserialize_dat, save_dat, DatVariant};
use crate::util::{
    GetId, ReadUnreal, UnrealCasts, UnrealReader, UnrealWriter, WriteUnreal, ASCF, DWORD, UVEC,
};
use r#macro::{ReadUnreal, WriteUnreal};
use std::thread;
use std::thread::JoinHandle;

impl From<(&Recipe, &mut L2GeneralStringTable)> for RecipeDat {
    fn from(value: (&Recipe, &mut L2GeneralStringTable)) -> Self {
        let (recipe, _table) = value;

        RecipeDat {
            name: (&recipe.name).into(),
            id: recipe.id.into(),
            recipe_item_id: recipe.recipe_item.into(),
            level: recipe.level,
            product_id: recipe.product.into(),
            product_count: recipe.product_count,
            show_tree: recipe.show_tree.to_u32_bool(),
            is_multiple_product: recipe.is_multiple_product.to_u32_bool(),
            mp_consume: recipe.mp_consume,
            success_rate: recipe.success_rate,
            materials: recipe
                .materials
                .iter()
                .map(|v| DatMaterial {
                    id: v.id.into(),
                    count: v.count,
                    recipe_id: v.recipe_id.0,
                })
                .collect::<Vec<DatMaterial>>()
                .into(),
        }
    }
}

impl Loader110 {
    pub fn serialize_recipes_to_binary(&mut self) -> JoinHandle<()> {
        let mut set_grp: Vec<RecipeDat> = vec![];

        for set in self.recipes.values() {
            set_grp.push((set, &mut self.game_data_name).into());
        }

        let set_grp_path = self
            .dat_paths
            .get(&"recipe.dat".to_string())
            .unwrap()
            .clone();

        thread::spawn(move || {
            save_dat(
                set_grp_path.path(),
                DatVariant::<(), RecipeDat>::Array(set_grp.to_vec()),
            )
            .unwrap();

            println!("Recipes Saved")
        })
    }

    pub fn load_recipes(&mut self) -> Result<(), ()> {
        let set_grp = deserialize_dat::<RecipeDat>(
            self.dat_paths
                .get(&"recipe.dat".to_string())
                .unwrap()
                .path(),
        )?;

        for v in set_grp {
            self.recipes.insert(
                v.id.into(),
                Recipe {
                    id: v.id.into(),
                    name: v.name.to_string(),
                    recipe_item: v.recipe_item_id.into(),
                    level: v.level,
                    product: v.product_id.into(),
                    product_count: v.product_count,
                    show_tree: v.show_tree == 1,
                    is_multiple_product: v.is_multiple_product == 1,
                    mp_consume: v.mp_consume,
                    success_rate: v.success_rate,
                    materials: v
                        .materials
                        .inner
                        .iter()
                        .map(|v| RecipeMaterial {
                            id: v.id.into(),
                            count: v.count,
                            recipe_id: v.recipe_id.into(),
                        })
                        .collect(),
                },
            );
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct DatMaterial {
    id: DWORD,
    count: DWORD,
    recipe_id: DWORD,
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct RecipeDat {
    name: ASCF,
    id: DWORD,
    recipe_item_id: DWORD,
    level: DWORD,

    product_id: DWORD,
    product_count: DWORD,

    show_tree: DWORD,
    is_multiple_product: DWORD,

    mp_consume: DWORD,
    success_rate: DWORD,
    materials: UVEC<DWORD, DatMaterial>,
}

impl GetId for RecipeDat {
    #[inline(always)]
    fn get_id(&self) -> u32 {
        self.id
    }
}
