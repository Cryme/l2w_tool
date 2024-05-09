use crate::backend::holder::FHashMap;
use crate::backend::entity_impl::item::{ItemAdditionalInfoAction, ItemDropInfoAction};
use crate::backend::{
    Backend, HandleAction,
};
use crate::data::ItemId;
use crate::entity::item::armor::Armor;
use crate::entity::CommonEntity;
use serde::{Deserialize, Serialize};
use crate::backend::entity_editor::{CommonEditorOps, CurrentEntity, EditParams, EntityEditParams, WindowParams};

pub type ArmorEditor = EntityEditParams<Armor, ItemId, ArmorAction, ()>;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum ArmorAction {
    #[default]
    None,
    RemoveSound(usize),
}

impl HandleAction for WindowParams<Armor, ItemId, ArmorAction, ()> {
    fn handle_action(&mut self) {
        let item = self;

        let mut action = item.action.write().unwrap();

        match *action {
            ArmorAction::RemoveSound(i) => {
                item.inner.item_sound.remove(i);
            }

            ArmorAction::None => {}
        }

        *action = ArmorAction::None;

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

impl EditParams {
    pub fn get_opened_armor_info(&self) -> Vec<(String, ItemId, bool)> {
        self.armor.get_opened_info()
    }

    pub fn open_armor(&mut self, id: ItemId, holder: &mut FHashMap<ItemId, Armor>) {
        for (i, q) in self.armor.opened.iter().enumerate() {
            if q.inner.initial_id == id {
                self.current_entity = CurrentEntity::Armor(i);

                return;
            }
        }

        if let Some(q) = holder.get(&id) {
            self.current_entity = CurrentEntity::Armor(self.armor.add(q.clone(), q.id()));
        }
    }

    pub fn set_current_armor(&mut self, index: usize) {
        if index < self.armor.opened.len() {
            self.current_entity = CurrentEntity::Armor(index);
        }
    }

    pub fn create_new_armor(&mut self) {
        self.current_entity = CurrentEntity::Armor(self.armor.add_new());
    }
}

impl Backend {
    pub fn filter_armor(&mut self) {
        self.entity_catalogs.armor.filter(&self.holders.game_data_holder.armor_holder);
    }

    pub fn save_armor_from_dlg(&mut self, id: ItemId) {
        if let CurrentEntity::Armor(index) = self.edit_params.current_entity {
            let new_entity = self.edit_params.armor.opened.get_mut(index).unwrap();

            if new_entity.inner.inner.id() != id {
                return;
            }

            new_entity.inner.initial_id = new_entity.inner.inner.id();

            let entity = new_entity.inner.inner.clone();

            self.save_armor_force(entity);
        }
    }

    pub(crate) fn save_armor_force(&mut self, v: Armor) {
        if let Some(vv) = self.holders.game_data_holder.armor_holder.get(&v.id()) {
            if *vv == v {
                return;
            }
        }
        self.set_changed();

        self.holders
            .game_data_holder
            .item_holder
            .insert(v.base_info.id, (&v).into());

        self.holders
            .game_data_holder
            .armor_holder
            .insert(v.base_info.id, v);

        self.filter_armor();
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub struct ArmorInfo {
    pub(crate) id: ItemId,
    pub(crate) name: String,
}

impl From<&Armor> for ArmorInfo {
    fn from(value: &Armor) -> Self {
        ArmorInfo {
            id: value.base_info.id,
            name: format!(
                "{} {}",
                value.base_info.name, value.base_info.additional_name
            ),
        }
    }
}
