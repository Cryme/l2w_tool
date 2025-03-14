use crate::backend::util::StringCow;
use crate::common::{EnsoulOptionId, ItemId};
use crate::entity::{CommonEntity, GetEditParams};
use rhai::{CustomType, TypeBuilder};
use serde::{Deserialize, Serialize};

impl GetEditParams<()> for EnsoulOption {
    fn edit_params(&self) {}
}

impl CommonEntity<EnsoulOptionId> for EnsoulOption {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn desc(&self) -> String {
        self.desc.clone()
    }

    fn id(&self) -> EnsoulOptionId {
        self.id
    }

    fn changed(&self) -> bool {
        self._changed
    }

    fn deleted(&self) -> bool {
        self._deleted
    }

    fn new(id: EnsoulOptionId) -> Self {
        EnsoulOption {
            id,
            option_type: 0,
            step: 0,
            name: "".to_string(),
            desc: "".to_string(),
            extraction_item_id: Default::default(),
            icon: "".into(),
            icon_panel: "".into(),

            _changed: false,
            _deleted: false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialEq, CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct EnsoulOption {
    pub id: EnsoulOptionId,
    pub option_type: u32,
    pub step: u32,
    pub name: String,
    pub desc: String,
    pub extraction_item_id: ItemId,
    pub icon: StringCow,
    pub icon_panel: StringCow,

    #[serde(skip)]
    pub _changed: bool,
    #[serde(skip)]
    pub _deleted: bool,
}
