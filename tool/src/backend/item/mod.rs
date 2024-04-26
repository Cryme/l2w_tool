use serde::{Deserialize, Serialize};

pub mod armor;
pub mod etc_item;
pub mod weapon;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum ItemAdditionalInfoAction {
    #[default]
    None,
    RemoveItem(usize),
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum ItemDropInfoAction {
    #[default]
    None,
    RemoveMesh(usize),
}
