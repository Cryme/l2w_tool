mod items;

use crate::backend::holder::HolderMapOps;
use crate::backend::Backend;
use crate::entity::item::armor::Armor;
use crate::entity::item::weapon::Weapon;
use rhai::{Dynamic, Engine};
use serde::Deserialize;
use crate::entity::CommonEntity;

#[derive(Debug, Deserialize)]
struct ChangedEntities {
    armor: Vec<Armor>,
    weapon: Vec<Weapon>,
}

impl Backend {
    pub fn run_script(&mut self, script: &str) -> String {
        let mut engine = Engine::new();

        items::reg(&mut engine);

        let mut changed_entities: ChangedEntities = ChangedEntities {
            armor: vec![],
            weapon: vec![],
        };

        let mut log = vec![];

        //Overloads
        {
            fn id_eq_1(lhs: u32, rhs: i64) -> bool {
                lhs as i64 == rhs
            }
            fn id_eq_2(lhs: i64, rhs: u32) -> bool {
                lhs == rhs as i64
            }
            engine.register_fn("==", id_eq_1);
            engine.register_fn("==", id_eq_2);
        }

        unsafe {
            let ptr: *const Backend = self;

            let log_ptr: *mut Vec<String> = &mut log;

            let changed_entities_ptr: *mut ChangedEntities = &mut changed_entities;

            let change_armor = move |x: Armor| {
                (*changed_entities_ptr).armor.push(x);
            };
            engine.register_fn("change_armor", change_armor);

            let change_weapon = move |x: Weapon| {
                (*changed_entities_ptr).weapon.push(x);
            };
            engine.register_fn("change_weapon", change_weapon);

            let armor_list = move || -> Dynamic {
                (*ptr)
                    .holders
                    .game_data_holder
                    .armor_holder
                    .values()
                    .map(|v| v.clone())
                    .collect::<Vec<_>>()
                    .into()
            };
            engine.register_fn("armor_list", armor_list);

            let weapon_list = move || -> Dynamic {
                (*ptr)
                    .holders
                    .game_data_holder
                    .weapon_holder
                    .values()
                    .map(|v| v.clone())
                    .collect::<Vec<_>>()
                    .into()
            };
            engine.register_fn("weapon_list", weapon_list);

            engine.on_print(move |x| {
                (*log_ptr).push(x.to_string());
            });
        }

        match engine.eval::<Dynamic>(script) {
            Ok(_) => {
                println!("{changed_entities:?}");

                for v in changed_entities.armor {
                    self.save_armor_force(v);
                }

                if log.is_empty() {
                    "Completed".to_string()
                } else {
                    let mut res = String::new();

                    for log in log {
                        res += &format!("{}\n", log);
                    }

                    res
                }
            }
            Err(err) => {
                format!("{:?}", err)
            }
        }
    }
}
