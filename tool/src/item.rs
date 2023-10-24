use crate::data::ItemId;

pub struct Item {
    pub(crate) id: ItemId,
    pub(crate) name: String,
    pub(crate) desc: String,
}
