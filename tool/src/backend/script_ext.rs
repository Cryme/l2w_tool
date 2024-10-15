use std::cell::RefCell;
use std::hash::Hash;
use std::rc::Rc;
use crate::backend::Backend;
use rhai::{CustomType, Engine, TypeBuilder};
use crate::backend::holder::HolderMapOps;
use crate::entity::item::armor::Armor;
use crate::entity::item::weapon::Weapon;

impl Backend {
    pub fn run_script(&mut self) {
        let mut engine = Engine::new();

        #[derive(Clone, CustomType)]
        #[rhai_type(extra = Self::build_extra)]
        struct ChangedEntities {
            armor: Vec<Armor>,
            weapon: Vec<Weapon>,
        }

        impl ChangedEntities {
            pub fn new() -> Self {
                Self {
                    armor: vec![],
                    weapon: vec![],
                }
            }

            fn build_extra(builder: &mut TypeBuilder<Self>) {
                builder
                    .with_name("ChangedEntities")
                    .with_fn("new_changed_entities", Self::new);
            }
        }

        engine.build_type::<ChangedEntities>();

        unsafe {
            let ptr: *const Backend = self;

            let armor_list = move || -> Vec<Armor> {
                (*ptr).holders.game_data_holder.armor_holder.values().map(|v| v.clone()).collect::<Vec<_>>()
            };

            let weapon_list = move || -> Vec<Weapon> {
                (*ptr).holders.game_data_holder.weapon_holder.values().map(|v| v.clone()).collect::<Vec<_>>()
            };

            engine.register_fn("armor_list", armor_list);
            engine.register_fn("weapon_list", weapon_list);

            let result = engine.eval::<ChangedEntities>(r#"
let changed_entities = new_changed_entities();
changed_entities.armor = armor_list();
changed_entities.weapon = weapon_list();

changed_entities
"#).unwrap();

            println!("{} {}", result.armor.len(), result.weapon.len());

        }
    }
}

