use serde::{Deserialize, Serialize};

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
