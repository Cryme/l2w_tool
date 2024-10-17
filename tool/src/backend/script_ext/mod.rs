mod items;

use crate::backend::holder::HolderMapOps;
use crate::backend::Backend;
use crate::common::ItemId;
use crate::entity::item::armor::Armor;
use crate::entity::item::etc_item::EtcItem;
use crate::entity::item::weapon::Weapon;
use crate::entity::CommonEntity;
use rhai::{Dynamic, Engine};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ChangedEntities {
    armor: Vec<Armor>,
    weapon: Vec<Weapon>,
    etc: Vec<EtcItem>,
}

impl Backend {
    pub fn run_script(&mut self, script: &str) -> String {
        let mut engine = Engine::new();

        items::reg(&mut engine);

        let mut changed_entities: ChangedEntities = ChangedEntities {
            armor: vec![],
            weapon: vec![],
            etc: vec![],
        };

        let mut log = vec![];

        //Eq Overloads
        {
            engine.register_fn("==", |lhs: u32, rhs: i64| -> bool { lhs as i64 == rhs });
            engine.register_fn("==", |lhs: i64, rhs: u32| -> bool { lhs == rhs as i64 });
            engine.register_fn("==", |lhs: ItemId, rhs: i64| -> bool {
                lhs.0 as i64 == rhs
            });
            engine.register_fn("==", |lhs: i64, rhs: ItemId| -> bool {
                lhs == rhs.0 as i64
            });
        }

        unsafe {
            let ptr: *const Backend = self;

            let log_ptr: *mut Vec<String> = &mut log;

            let changed_entities_ptr: *mut ChangedEntities = &mut changed_entities;

            engine.register_fn("save", move |x: Armor| {
                (*changed_entities_ptr).armor.push(x);
            });
            engine.register_fn("save", move |x: Weapon| {
                (*changed_entities_ptr).weapon.push(x);
            });
            engine.register_fn("save", move |x: EtcItem| {
                (*changed_entities_ptr).etc.push(x);
            });

            engine.register_fn("armor_list", move || -> Dynamic {
                (*ptr)
                    .holders
                    .game_data_holder
                    .armor_holder
                    .values().cloned()
                    .collect::<Vec<_>>()
                    .into()
            });
            engine.register_fn("weapon_list", move || -> Dynamic {
                (*ptr)
                    .holders
                    .game_data_holder
                    .weapon_holder
                    .values().cloned()
                    .collect::<Vec<_>>()
                    .into()
            });
            engine.register_fn("etc_list", move || -> Dynamic {
                (*ptr)
                    .holders
                    .game_data_holder
                    .etc_item_holder
                    .values().cloned()
                    .collect::<Vec<_>>()
                    .into()
            });

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
