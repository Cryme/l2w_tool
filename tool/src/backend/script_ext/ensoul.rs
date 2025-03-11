use crate::backend::holder::HolderMapOps;
use crate::backend::script_ext::ChangedEntities;
use crate::backend::Backend;
use crate::common::{EnsoulOptionId};
use crate::entity::ensoul_option::EnsoulOption;
use rhai::{Dynamic, Engine, TypeBuilder};

impl EnsoulOption {
    /// Additional API's
    pub fn build_extra(builder: &mut TypeBuilder<Self>) {
        builder.on_print(|v| format!("EnsoulOption(id: {}, name: {})", v.id.0, v.name));
    }
}
impl EnsoulOptionId {
    /// Additional API's
    pub fn build_extra(builder: &mut TypeBuilder<Self>) {
        builder.on_print(|v| format!("EnsoulOptionId({})", v.0));
    }
}

pub fn proceed(backend: &mut Backend, ensoul_option: Vec<EnsoulOption>) {
    for mut v in ensoul_option {
        v._changed = true;
        backend.editors.force_update_ensoul_option(&v);
        backend.save_ensoul_option_force(v);
    }
}

pub fn reg(engine: &mut Engine, changed_entities_ptr: *mut ChangedEntities, ptr: *const Backend) {
    //Eq Overloads
    {
        engine.register_fn("==", |lhs: EnsoulOptionId, rhs: i64| -> bool {
            lhs.0 as i64 == rhs
        });
        engine.register_fn("==", |lhs: i64, rhs: EnsoulOptionId| -> bool {
            lhs == rhs.0 as i64
        });
    }

    unsafe {
        engine.register_fn("save", move |x: EnsoulOption| {
            (*changed_entities_ptr).ensoul_option.push(x);
        });
        engine.register_fn("delete", move |mut x: EnsoulOption| {
            x._deleted = true;

            (*changed_entities_ptr).ensoul_option.push(x);
        });

        engine.register_fn("ensoul_list", move || -> Dynamic {
            (*ptr)
                .holders
                .game_data_holder
                .ensoul_option_holder
                .values()
                .cloned()
                .collect::<Vec<_>>()
                .into()
        });
    }

    engine.build_type::<EnsoulOption>();
    engine.build_type::<EnsoulOptionId>();
    engine.register_fn("set_id", |v: &mut EnsoulOption, id: i64| {
        v.id.0 = id as u32;
    });
}
