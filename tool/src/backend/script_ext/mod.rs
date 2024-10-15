mod items;

use std::hash::Hash;
use crate::backend::Backend;
use rhai::plugin::*;
use rhai::{Dynamic, Engine};
use crate::backend::holder::HolderMapOps;

impl Backend {
    pub fn run_script(&mut self) {
        let mut engine = Engine::new();

        items::reg(&mut engine);

        unsafe {
            let ptr: *const Backend = self;

            let armor_list = move || -> Dynamic {
                (*ptr).holders.game_data_holder.armor_holder.values().map(|v| v.clone()).collect::<Vec<_>>().into()
            };

            let weapon_list = move || -> Dynamic {
                (*ptr).holders.game_data_holder.weapon_holder.values().map(|v| Dynamic::from(v.clone())).collect::<Vec<Dynamic>>().into()
            };

            engine.register_fn("armor_list", armor_list);
            engine.register_fn("weapon_list", weapon_list);
        }


        let result: Dynamic = engine.eval(r#"
let armor = armor_list();
let weapon = weapon_list();

print(armor[0].base_info.name);
"#).unwrap();

        // println!("{} {}", result.armor.len(), result.weapon.len());
    }
}
