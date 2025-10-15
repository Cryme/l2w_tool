use crate::backend::editor::entity::{CommonEditorOps, EntityEditParams};
use crate::backend::editor::{CurrentEntity, EditParamsCommonOps, Editors, WindowParams};
use crate::backend::entity_catalog::EntityInfo;
use crate::backend::entity_impl::item::{ItemAdditionalInfoAction, ItemDropInfoAction};
use crate::backend::holder::{FHashMap, HolderMapOps};
use crate::backend::{Backend, HandleAction};
use crate::common::ItemId;
use crate::entity::item::etc_item::EtcItem;
use crate::entity::{CommonEntity, GameEntityT};
use serde::{Deserialize, Serialize};

pub type EtcItemEditor = EntityEditParams<EtcItem, ItemId, EtcItemAction, ()>;

impl HandleAction for WindowParams<EtcItem, ItemId, EtcItemAction, ()> {
    fn handle_action(&mut self) {
        let item = self;

        let mut action = item.action.write().unwrap();

        match *action {
            EtcItemAction::RemoveMesh(i) => {
                item.inner.mesh_info.remove(i);
            }
            EtcItemAction::RemoveStoneOption(i) => {
                if let Some(s) = &mut item.inner.ensoul_stone {
                    s.options.remove(i);
                }
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
    RemoveStoneOption(usize),
}

impl Editors {
    pub fn force_update_etc_item(&mut self, item: &EtcItem) {
        if item._deleted {
            self.close_if_opened(GameEntityT::EtcItem(item.id()));
        } else if let Some(v) = self
            .etc_items
            .opened
            .iter_mut()
            .find(|v| v.inner.inner.id() == item.id())
        {
            v.inner.inner = item.clone();
        }
    }

    pub fn get_opened_etc_items_info(&self) -> Vec<(String, ItemId, bool)> {
        self.etc_items.get_opened_info()
    }

    pub fn open_etc_item(&mut self, id: ItemId, holder: &mut FHashMap<ItemId, EtcItem>) {
        for (i, q) in self.etc_items.opened.iter().enumerate() {
            if q.inner.initial_id == id {
                self.current_entity = CurrentEntity::EtcItem(i);

                return;
            }
        }

        if let Some(q) = holder.get(&id) {
            self.current_entity =
                CurrentEntity::EtcItem(self.etc_items.add(q.clone(), q.id(), false));
        }
    }

    pub fn set_current_etc_item(&mut self, index: usize) {
        if index < self.etc_items.opened.len() {
            self.current_entity = CurrentEntity::EtcItem(index);
        }
    }

    pub fn create_new_etc_item(&mut self) {
        self.current_entity = CurrentEntity::EtcItem(self.etc_items.add_new());
    }
}

impl Backend {
    pub fn filter_etc_items(&mut self) {
        self.entity_catalogs.etc_item.filter(
            &self.holders.game_data_holder.etc_item_holder,
            self.entity_catalogs.filter_mode,
        );
    }

    pub fn save_etc_item_from_dlg(&mut self, id: ItemId) {
        if let CurrentEntity::EtcItem(index) = self.editors.current_entity {
            let new_entity = self.editors.etc_items.opened.get_mut(index).unwrap();

            if new_entity.inner.inner.id() != id {
                return;
            }

            new_entity.inner.initial_id = new_entity.inner.inner.id();

            let entity = new_entity.inner.inner.clone();

            new_entity.on_save();

            self.save_etc_item_force(entity);
        }
    }

    pub(crate) fn save_etc_item_force(&mut self, mut v: EtcItem) {
        if let Some(vv) = self.holders.game_data_holder.etc_item_holder.get(&v.id())
            && *vv == v {
                return;
            }
        v._changed = true;

        if self
            .holders
            .game_data_holder
            .armor_holder
            .remove(&v.base_info.id)
            .is_some()
        {
            self.editors
                .close_if_opened(GameEntityT::Armor(v.base_info.id));
            self.filter_armor();
        }
        if self
            .holders
            .game_data_holder
            .weapon_holder
            .remove(&v.base_info.id)
            .is_some()
        {
            self.editors
                .close_if_opened(GameEntityT::EtcItem(v.base_info.id));
            self.filter_weapons();
        }

        self.holders
            .game_data_holder
            .item_holder
            .insert(v.base_info.id, (&v).into());
        self.holders
            .game_data_holder
            .etc_item_holder
            .insert(v.base_info.id, v);

        self.filter_etc_items();
        self.check_for_unwrote_changed();
    }
}

impl From<&EtcItem> for EntityInfo<EtcItem, ItemId> {
    fn from(value: &EtcItem) -> Self {
        EntityInfo::new(
            &format!(
                "ID: {}\n{} {}",
                value.base_info.id.0, value.base_info.name.ru, value.base_info.additional_name.ru
            ),
            value,
        )
    }
}
