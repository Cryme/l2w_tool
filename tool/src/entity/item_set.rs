use crate::data::{ItemId, ItemSetId};
use crate::entity::{CommonEntity, GetEditParams};
use serde::{Deserialize, Serialize};
use std::fmt::Write;

impl GetEditParams<()> for ItemSet {
    fn edit_params(&self) {}
}

impl CommonEntity<ItemSetId> for ItemSet {
    fn name(&self) -> String {
        self.id.0.to_string()
    }

    fn desc(&self) -> String {
        self.base_descriptions
            .iter()
            .enumerate()
            .fold(String::new(), |mut res, v| {
                let _ = writeln!(res, "{}: {}", v.0 + 1, v.1);

                res
            })
    }

    fn id(&self) -> ItemSetId {
        self.id
    }

    fn changed(&self) -> bool {
        self._changed
    }

    fn deleted(&self) -> bool {
        self._deleted
    }

    fn new(id: ItemSetId) -> Self {
        Self {
            id,
            base_items: vec![],
            base_descriptions: vec![],
            additional_items: vec![],
            additional_descriptions: vec![],
            unk1: 0,
            unk2: 0,
            enchant_info: vec![],

            _changed: false,
            _deleted: false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct ItemSetEnchantInfo {
    pub(crate) enchant_level: u32,
    pub(crate) enchant_description: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Default)]
pub struct ItemSet {
    pub(crate) id: ItemSetId,

    pub(crate) base_items: Vec<Vec<ItemId>>,
    pub(crate) base_descriptions: Vec<String>,

    pub(crate) additional_items: Vec<Vec<ItemId>>,
    pub(crate) additional_descriptions: Vec<String>,

    pub(crate) unk1: u32,
    pub(crate) unk2: u32,

    pub(crate) enchant_info: Vec<ItemSetEnchantInfo>,

    pub _changed: bool,
    pub _deleted: bool,
}
