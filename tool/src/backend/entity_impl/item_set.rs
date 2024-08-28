use crate::backend::editor::entity::{CommonEditorOps, EntityEditParams};
use crate::backend::editor::{CurrentEntity, EditParamsCommonOps, Editors, WindowParams};
use crate::backend::entity_catalog::EntityInfo;
use crate::backend::holder::{FHashMap, HolderMapOps};
use crate::backend::{Backend, HandleAction};
use crate::common::{ItemId, ItemSetId};
use crate::entity::item_set::ItemSet;
use crate::entity::CommonEntity;
use serde::{Deserialize, Serialize};

pub type ItemSetEditor = EntityEditParams<ItemSet, ItemSetId, ItemSetAction, ()>;

impl HandleAction for WindowParams<ItemSet, ItemSetId, ItemSetAction, ()> {
    fn handle_action(&mut self) {
        let item = self;

        let mut action = item.action.write().unwrap();

        match *action {
            ItemSetAction::AddBaseSetLevel => {
                item.inner
                    .base_descriptions
                    .push("New Description".to_string());
            }

            ItemSetAction::RemoveBaseSetLevel(i) => {
                item.inner.base_descriptions.remove(i);
            }

            ItemSetAction::AddBaseItemGroup => {
                item.inner.base_items.push(vec![ItemId(1)]);
            }

            ItemSetAction::RemoveBaseItemGroup(i) => {
                item.inner.base_items.remove(i);
            }

            ItemSetAction::AddBaseGroupItem(i) => {
                item.inner.base_items[i].push(ItemId(1));
            }

            ItemSetAction::RemoveBaseGroupItem(i, ii) => {
                item.inner.base_items[i].remove(ii);
            }

            ItemSetAction::AddAdditionalSetLevel => {
                item.inner
                    .additional_descriptions
                    .push("New Description".to_string());
            }

            ItemSetAction::RemoveAdditionalSetLevel(i) => {
                item.inner.additional_descriptions.remove(i);
            }

            ItemSetAction::AddAdditionalItemGroup => {
                item.inner.additional_items.push(vec![ItemId(1)]);
            }

            ItemSetAction::RemoveAdditionalItemGroup(i) => {
                item.inner.additional_items.remove(i);
            }

            ItemSetAction::AddAdditionalGroupItem(i) => {
                item.inner.additional_items[i].push(ItemId(1));
            }

            ItemSetAction::RemoveAdditionalGroupItem(i, ii) => {
                item.inner.additional_items[i].remove(ii);
            }

            ItemSetAction::None => {}
        }

        *action = ItemSetAction::None;
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum ItemSetAction {
    #[default]
    None,

    AddBaseSetLevel,
    RemoveBaseSetLevel(usize),

    AddBaseItemGroup,
    RemoveBaseItemGroup(usize),

    AddBaseGroupItem(usize),
    RemoveBaseGroupItem(usize, usize),

    AddAdditionalSetLevel,
    RemoveAdditionalSetLevel(usize),

    AddAdditionalItemGroup,
    RemoveAdditionalItemGroup(usize),

    AddAdditionalGroupItem(usize),
    RemoveAdditionalGroupItem(usize, usize),
}

impl Editors {
    pub fn get_opened_item_sets_info(&self) -> Vec<(String, ItemSetId, bool)> {
        self.item_sets.get_opened_info()
    }

    pub fn open_item_set(&mut self, id: ItemSetId, holder: &mut FHashMap<ItemSetId, ItemSet>) {
        for (i, q) in self.item_sets.opened.iter().enumerate() {
            if q.inner.initial_id == id {
                self.current_entity = CurrentEntity::ItemSet(i);

                return;
            }
        }

        if let Some(q) = holder.get(&id) {
            self.current_entity =
                CurrentEntity::ItemSet(self.item_sets.add(q.clone(), q.id(), false));
        }
    }

    pub fn set_current_item_set(&mut self, index: usize) {
        if index < self.item_sets.opened.len() {
            self.current_entity = CurrentEntity::ItemSet(index);
        }
    }

    pub fn create_new_item_set(&mut self) {
        self.current_entity = CurrentEntity::ItemSet(self.item_sets.add_new());
    }
}

impl Backend {
    pub fn filter_item_sets(&mut self) {
        self.entity_catalogs.item_set.filter(
            &self.holders.game_data_holder.item_set_holder,
            self.entity_catalogs.filter_mode,
        );
    }

    pub fn save_item_set_from_dlg(&mut self, id: ItemSetId) {
        if let CurrentEntity::ItemSet(index) = self.editors.current_entity {
            let new_entity = self.editors.item_sets.opened.get_mut(index).unwrap();

            if new_entity.inner.inner.id() != id {
                return;
            }

            new_entity.inner.initial_id = new_entity.inner.inner.id;

            let entity = new_entity.inner.inner.clone();

            new_entity.on_save();

            self.save_item_set_force(entity);
        }
    }

    pub(crate) fn save_item_set_force(&mut self, mut v: ItemSet) {
        if let Some(vv) = self.holders.game_data_holder.item_set_holder.get(&v.id) {
            if *vv == v {
                return;
            }
        }
        v._changed = true;

        self.holders
            .game_data_holder
            .item_set_holder
            .insert(v.id, v);

        self.filter_item_sets();
        self.check_for_unwrote_changed();
    }
}

impl From<&ItemSet> for EntityInfo<ItemSet, ItemSetId> {
    fn from(value: &ItemSet) -> Self {
        EntityInfo::new(&format!("ID: {}\n", value.id.0), value)
    }
}
