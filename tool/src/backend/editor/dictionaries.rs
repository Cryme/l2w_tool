use crate::backend::holder::{DictEditItem, DictItem, HolderMapOps};
use crate::backend::util::is_in_range;
use crate::backend::Backend;
use crate::entity::Dictionary;
use serde::Serialize;
use std::hash::Hash;
use std::ops::{Index, IndexMut};

#[derive(Default)]
pub struct DictEditor<ID: Hash + Eq + Ord + Copy, T: Clone + Ord + Eq + Serialize + Default> {
    pub items: Vec<DictEditItem<ID, T>>,
    pub changed_count: usize,
    pub filtered_indexes: Vec<usize>,
    pub search: String,
}

impl DictEditor<u32, String> {
    pub fn add_new(&mut self) {
        let id = if let Some(v) = self.items.last() {
            v.id + 1
        } else {
            0
        };

        self.changed_count += 1;

        self.search = format!("r:{id}");

        self.items.push(DictEditItem::new(
            id,
            ("-- NEW --".to_string(), "-- NEW --".to_string()).into(),
        ));

        self.apply_search()
    }
}

impl<ID: Hash + Eq + Ord + Copy, T: Clone + Ord + Eq + Serialize + Default> DictEditor<ID, T> {
    pub fn new(items: Vec<DictItem<ID, T>>) -> Self {
        Self {
            items: items.iter().map(|v| v.into()).collect(),
            changed_count: 0,
            filtered_indexes: items.iter().enumerate().map(|(i, _)| i).collect(),
            search: String::new(),
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

pub trait DictEditorOps<K: Hash + Eq + Ord + Copy, V: Clone + Ord + Eq + Serialize + Default> {
    #[allow(unused)]
    fn items(&self) -> &Vec<DictEditItem<K, V>>;
    fn items_mut(&mut self) -> &mut Vec<DictEditItem<K, V>>;

    fn set_changed_count(&mut self, changed_count: usize);
    fn apply_search(&mut self);
}

impl DictEditorOps<u32, String> for DictEditor<u32, String> {
    fn items(&self) -> &Vec<DictEditItem<u32, String>> {
        &self.items
    }
    fn items_mut(&mut self) -> &mut Vec<DictEditItem<u32, String>> {
        &mut self.items
    }

    fn set_changed_count(&mut self, changed_count: usize) {
        self.changed_count = changed_count;
    }

    fn apply_search(&mut self) {
        if let Some(range) = self.search.strip_prefix("r:") {
            self.filtered_indexes = self
                .items
                .iter()
                .enumerate()
                .filter(|(_, v)| is_in_range(range, v.id))
                .map(|(i, _)| i)
                .collect();
        } else {
            let search = self.search.to_lowercase();
            self.filtered_indexes = self
                .items
                .iter()
                .enumerate()
                .filter(|(_, v)| v.item.lowered_contains(&search))
                .map(|(i, _)| i)
                .collect();
        }
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
