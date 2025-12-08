use serde::{Deserialize, Serialize};

pub mod armor;
pub mod etc_item;
pub mod weapon;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub enum ItemAdditionalInfoAction {
    #[default]
    None,
    RemoveItem(usize),
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub enum ItemDropInfoAction {
    #[default]
    None,
    RemoveMesh(usize),
}
