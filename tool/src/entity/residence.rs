use crate::data::ResidenceId;
use crate::entity::{CommonEntity, GetEditParams};
use serde::{Deserialize, Serialize};

impl GetEditParams<()> for Residence {
    fn edit_params(&self) {}
}

impl CommonEntity<ResidenceId> for Residence {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn desc(&self) -> String {
        self.desc.clone()
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
            name: "Residence".to_string(),
            desc: "Desc".to_string(),

            territory: "".to_string(),
            mark: "".to_string(),
            mark_grey: "".to_string(),
            flag_icon: "".to_string(),
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
    pub name: String,
    pub desc: String,
    pub territory: String,
    pub mark: String,
    pub mark_grey: String,
    pub flag_icon: String,
    pub merc_name: String,
    pub region_id: u16,

    #[serde(skip)]
    pub _changed: bool,
    #[serde(skip)]
    pub _deleted: bool,
}
