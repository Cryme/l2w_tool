use crate::backend::item::{ItemAdditionalInfoAction, ItemDropInfoAction};
use crate::backend::{
    Backend, CommonEditorOps, CurrentEntity, EditParams, EntityEditParams, HandleAction,
};
use crate::data::ItemId;
use crate::entity::item::etc_item::EtcItem;
use crate::entity::CommonEntity;
use crate::holder::FHashMap;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub type EtcItemEditor = EntityEditParams<EtcItem, ItemId, EtcItemAction, ()>;

impl HandleAction for EtcItemEditor {
    fn handle_action(&mut self, index: usize) {
        let item = &mut self.opened[index];

        let mut action = item.action.write().unwrap();

        match *action {
            EtcItemAction::RemoveMesh(i) => {
                item.inner.mesh_info.remove(i);
            }

            EtcItemAction::None => {}
        }

        *action = EtcItemAction::None;

        {
            let mut action = item.inner.base_info.additional_info.action.write().unwrap();
            match *action {
                ItemAdditionalInfoAction::RemoveItem(v) => {
                    item.inner
                        .base_info
                        .additional_info
                        .inner
                        .include_items
                        .remove(v);
                }

                ItemAdditionalInfoAction::None => {}
            }

            *action = ItemAdditionalInfoAction::None;
        }

        {
            let mut action = item.inner.base_info.drop_info.action.write().unwrap();
            match *action {
                ItemDropInfoAction::RemoveMesh(v) => {
                    item.inner
                        .base_info
                        .drop_info
                        .inner
                        .drop_mesh_info
                        .remove(v);
                }

                ItemDropInfoAction::None => {}
            }

            *action = ItemDropInfoAction::None;
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum EtcItemAction {
    #[default]
    None,
    RemoveMesh(usize),
}

impl EditParams {
    pub fn get_opened_etc_items_info(&self) -> Vec<(String, ItemId)> {
        self.etc_items.get_opened_info()
    }

    pub fn open_etc_item(&mut self, id: ItemId, holder: &mut FHashMap<ItemId, EtcItem>) {
        for (i, q) in self.etc_items.opened.iter().enumerate() {
            if q.initial_id == id {
                self.current_entity = CurrentEntity::EtcItem(i);

                return;
            }
        }

        if let Some(q) = holder.get(&id) {
            self.current_entity = CurrentEntity::EtcItem(self.etc_items.add(q.clone(), q.id()));
        }
    }

    pub fn set_current_etc_item(&mut self, index: usize) {
        if index < self.etc_items.opened.len() {
            self.current_entity = CurrentEntity::EtcItem(index);
        }
    }

    pub fn close_etc_item(&mut self, index: usize) {
        self.etc_items.opened.remove(index);

        if let CurrentEntity::EtcItem(curr_index) = self.current_entity {
            if self.etc_items.opened.is_empty() {
                self.find_opened_entity();
            } else if curr_index >= index {
                self.current_entity = CurrentEntity::EtcItem(curr_index.max(1) - 1)
            }
        }
    }

    pub fn create_new_etc_item(&mut self) {
        self.current_entity = CurrentEntity::EtcItem(self.etc_items.add_new());
    }
}

impl Backend {
    pub fn filter_etc_items(&mut self) {
        let s = self.filter_params.etc_item_filter_string.to_lowercase();

        let fun: Box<dyn Fn(&&EtcItem) -> bool> = if s.is_empty() {
            Box::new(|_: &&EtcItem| true)
        } else if let Ok(id) = u32::from_str(&s) {
            Box::new(move |v: &&EtcItem| v.base_info.id == ItemId(id))
        } else {
            Box::new(move |v: &&EtcItem| {
                v.base_info.name.to_lowercase().contains(&s)
                    || v.base_info.additional_name.to_lowercase().contains(&s)
            })
        };

        self.filter_params.etc_item_catalog = self
            .holders
            .game_data_holder
            .etc_item_holder
            .values()
            .filter(fun)
            .map(EtcItemInfo::from)
            .collect();

        self.filter_params
            .etc_item_catalog
            .sort_by(|a, b| a.id.cmp(&b.id))
    }

    pub fn save_etc_item_from_dlg(&mut self, id: ItemId) {
        if let CurrentEntity::EtcItem(index) = self.edit_params.current_entity {
            let new_entity = self.edit_params.etc_items.opened.get(index).unwrap();

            if new_entity.inner.id() != id {
                return;
            }

            self.save_etc_item_force(new_entity.inner.clone());
        }
    }

    pub(crate) fn save_etc_item_force(&mut self, v: EtcItem) {
        self.holders
            .game_data_holder
            .item_holder
            .insert(v.base_info.id, (&v).into());

        self.holders
            .game_data_holder
            .etc_item_holder
            .insert(v.base_info.id, v);

        self.filter_etc_items();
    }
}

pub struct EtcItemInfo {
    pub(crate) id: ItemId,
    pub(crate) name: String,
}

impl From<&EtcItem> for EtcItemInfo {
    fn from(value: &EtcItem) -> Self {
        EtcItemInfo {
            id: value.base_info.id,
            name: format!(
                "{} {}",
                value.base_info.name, value.base_info.additional_name
            ),
        }
    }
}
