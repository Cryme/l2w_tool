use crate::backend::util::{Localized, StringCow};
use crate::common::{EnsoulOptionId, ItemId};
use crate::entity::{CommonEntity, GetEditParams};
use rhai::{CustomType, TypeBuilder};
use serde::{Deserialize, Serialize};

impl GetEditParams<()> for EnsoulOption {
    fn edit_params(&self) {}
}

impl CommonEntity<EnsoulOptionId> for EnsoulOption {
    fn name(&self) -> String {
        self.name.ru.clone()
    }

    fn desc(&self) -> String {
        self.desc.ru.clone()
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
            icon: "".into(),
            icon_panel: "".into(),

            _changed: false,
            _deleted: false,

            ..Default::default()
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialEq, CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct EnsoulOption {
    pub id: EnsoulOptionId,
    pub option_type: u32,
    pub step: u32,
    pub name: Localized<String>,
    pub desc: Localized<String>,
    pub extraction_item_id: ItemId,
    pub icon: StringCow,
    pub icon_panel: StringCow,

    #[serde(skip)]
    pub _changed: bool,
    #[serde(skip)]
    pub _deleted: bool,
}
