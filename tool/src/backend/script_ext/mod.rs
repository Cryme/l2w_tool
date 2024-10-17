mod ensoul;
mod items;

use crate::backend::Backend;
use crate::entity::ensoul_option::EnsoulOption;
use crate::entity::item::armor::Armor;
use crate::entity::item::etc_item::EtcItem;
use crate::entity::item::weapon::Weapon;
use rhai::{Dynamic, Engine};
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
struct ChangedEntities {
    armor: Vec<Armor>,
    weapon: Vec<Weapon>,
    etc: Vec<EtcItem>,

    ensoul_option: Vec<EnsoulOption>,
}

impl Backend {
    pub fn run_script(&mut self, script: &str) -> String {
        let mut engine = Engine::new();

        let mut changed_entities: ChangedEntities = ChangedEntities::default();

        let mut log = vec![];

        //Eq Overloads
        {
            engine.register_fn("==", |lhs: u32, rhs: i64| -> bool { lhs as i64 == rhs });
            engine.register_fn("==", |lhs: i64, rhs: u32| -> bool { lhs == rhs as i64 });
        }

        unsafe {
            let ptr: *const Backend = self;

            let changed_entities_ptr: *mut ChangedEntities = &mut changed_entities;

            items::reg(&mut engine, changed_entities_ptr, ptr);
            ensoul::reg(&mut engine, changed_entities_ptr, ptr);

            let log_ptr: *mut Vec<String> = &mut log;

            engine.on_print(move |x| {
                (*log_ptr).push(x.to_string());
            });
        }

        match engine.eval::<Dynamic>(script) {
            Ok(_) => {
                for mut v in changed_entities.armor {
                    v._changed = true;
                    self.editors.force_update_armor(&v);
                    self.save_armor_force(v);
                }
                for mut v in changed_entities.weapon {
                    v._changed = true;
                    self.editors.force_update_weapon(&v);
                    self.save_weapon_force(v);
                }
                for mut v in changed_entities.etc {
                    v._changed = true;
                    self.editors.force_update_etc_item(&v);
                    self.save_etc_item_force(v);
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
