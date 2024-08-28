use crate::backend::holder::{DictEditItem, DictItem, HolderMapOps};
use crate::backend::Backend;
use crate::entity::Dictionary;
use std::hash::Hash;
use std::ops::{Index, IndexMut};

#[derive(Default)]
pub struct DictEditor<ID: Hash + Eq + Ord + Copy, T: Clone + Ord + Eq> {
    pub items: Vec<DictEditItem<ID, T>>,
    pub changed_count: usize,
}

impl DictEditor<u32, String> {
    pub fn add_new(&mut self) {
        let id = if let Some(v) = self.items.last() {
            v.id + 1
        } else {
            0
        };

        self.changed_count += 1;
        self.items
            .push(DictEditItem::new(id, "-- NEW --".to_string()));
    }
}

impl<ID: Hash + Eq + Ord + Copy, T: Clone + Ord + Eq> DictEditor<ID, T> {
    pub fn new(items: Vec<DictItem<ID, T>>) -> Self {
        Self {
            items: items.iter().map(|v| v.into()).collect(),
            changed_count: 0,
        }
    }

    pub fn inc_changed(&mut self) {
        self.changed_count += 1;
    }

    pub fn dec_changed(&mut self) {
        self.changed_count -= 1;
    }

    pub fn changed(&self) -> bool {
        self.changed_count > 0
    }
}

#[derive(Default)]
pub struct DictEditors {
    pub system_strings: DictEditor<u32, String>,
    pub npc_strings: DictEditor<u32, String>,
}

pub trait DictEditorOps<K: Hash + Eq + Ord + Copy, V: Clone + Ord + Eq> {
    fn items(&self) -> &Vec<DictEditItem<K, V>>;
    fn items_mut(&mut self) -> &mut Vec<DictEditItem<K, V>>;

    fn set_changed_count(&mut self, changed_count: usize);
}

impl<K: Hash + Eq + Ord + Copy, V: Clone + Ord + Eq> DictEditorOps<K, V> for DictEditor<K, V> {
    fn items(&self) -> &Vec<DictEditItem<K, V>> {
        &self.items
    }
    fn items_mut(&mut self) -> &mut Vec<DictEditItem<K, V>> {
        &mut self.items
    }

    fn set_changed_count(&mut self, changed_count: usize) {
        self.changed_count = changed_count;
    }
}

impl Index<Dictionary> for DictEditors {
    type Output = dyn DictEditorOps<u32, String>;

    fn index(&self, entity: Dictionary) -> &Self::Output {
        match entity {
            Dictionary::SystemStrings => &self.system_strings,
            Dictionary::NpcStrings => &self.npc_strings,
        }
    }
}

impl IndexMut<Dictionary> for DictEditors {
    fn index_mut(&mut self, entity: Dictionary) -> &mut Self::Output {
        match entity {
            Dictionary::SystemStrings => &mut self.system_strings,
            Dictionary::NpcStrings => &mut self.npc_strings,
        }
    }
}

impl Backend {
    pub fn fill_system_strings_editor(&mut self) {
        let mut strings: Vec<_> = self
            .holders
            .game_data_holder
            .system_strings
            .values()
            .cloned()
            .collect();

        strings.sort_by(|a, b| a.id.cmp(&b.id));

        self.editors.dictionaries.system_strings = DictEditor::new(strings);
    }

    pub fn fill_npc_strings_editor(&mut self) {
        let mut strings: Vec<_> = self
            .holders
            .game_data_holder
            .npc_strings
            .values()
            .cloned()
            .collect();

        strings.sort_by(|a, b| a.id.cmp(&b.id));

        self.editors.dictionaries.npc_strings = DictEditor::new(strings);
    }
}
