use crate::backend::util::{Localized, StringCow};
use crate::common::ResidenceId;
use crate::entity::{CommonEntity, GetEditParams};
use serde::{Deserialize, Serialize};

impl GetEditParams<()> for Residence {
    fn edit_params(&self) {}
}

impl CommonEntity<ResidenceId> for Residence {
    fn name(&self) -> String {
        self.name.ru.clone()
    }

    fn desc(&self) -> String {
        self.desc.ru.clone()
    }

    fn id(&self) -> ResidenceId {
        self.id
    }

    fn changed(&self) -> bool {
        self._changed
    }

    fn deleted(&self) -> bool {
        self._deleted
    }

    fn new(id: ResidenceId) -> Self {
        Residence {
            id,
            name: ("Новое Владение".to_string(), "New Residence".to_string()).into(),
            desc: ("Новое Описание".to_string(), "New Description".to_string()).into(),

            territory: ("".to_string(), "".to_string()).into(),
            mark: "".into(),
            mark_grey: "".into(),
            flag_icon: "".into(),
            merc_name: "".to_string(),
            region_id: Default::default(),
            _changed: false,
            _deleted: false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct Residence {
    pub id: ResidenceId,
    pub name: Localized<String>,
    pub desc: Localized<String>,
    pub territory: Localized<String>,
    pub mark: StringCow,
    pub mark_grey: StringCow,
    pub flag_icon: StringCow,
    pub merc_name: String,
    pub region_id: u16,

    #[serde(skip)]
    pub _changed: bool,
    #[serde(skip)]
    pub _deleted: bool,
}
