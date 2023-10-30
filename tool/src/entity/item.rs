use crate::data::ItemId;

#[derive(Clone)]
pub struct Item {
    pub(crate) id: ItemId,
    pub(crate) name: String,
    pub(crate) desc: String,
}
