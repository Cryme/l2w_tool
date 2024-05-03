use crate::backend::{
    Backend, CommonEditorOps, CurrentEntity, EditParams, EntityEditParams, HandleAction,
};
use crate::data::RegionId;
use crate::entity::region::Region;
use crate::entity::CommonEntity;
use crate::holder::FHashMap;
use std::str::FromStr;

pub type RegionEditor = EntityEditParams<Region, RegionId, (), ()>;

impl HandleAction for RegionEditor {
    fn handle_action(&mut self, _index: usize) {}
}

impl EditParams {
    pub fn get_opened_region_info(&self) -> Vec<(String, RegionId)> {
        self.regions.get_opened_info()
    }

    pub fn open_region(&mut self, id: RegionId, holder: &mut FHashMap<RegionId, Region>) {
        for (i, q) in self.regions.opened.iter().enumerate() {
            if q.initial_id == id {
                self.current_entity = CurrentEntity::Region(i);

                return;
            }
        }

        if let Some(q) = holder.get(&id) {
            self.current_entity = CurrentEntity::Region(self.regions.add(q.clone(), q.id()));
        }
    }

    pub fn set_current_region(&mut self, index: usize) {
        if index < self.regions.opened.len() {
            self.current_entity = CurrentEntity::Region(index);
        }
    }

    pub fn close_region(&mut self, index: usize) {
        self.regions.opened.remove(index);

        if let CurrentEntity::Region(curr_index) = self.current_entity {
            if self.regions.opened.is_empty() {
                self.find_opened_entity();
            } else if curr_index >= index {
                self.current_entity = CurrentEntity::Region(curr_index.max(1) - 1)
            }
        }
    }

    pub fn create_new_region(&mut self) {
        self.current_entity = CurrentEntity::Region(self.regions.add_new());
    }
}

impl Backend {
    pub fn filter_regions(&mut self) {
        let s = self.filter_params.region_filter_string.to_lowercase();

        let fun: Box<dyn Fn(&&Region) -> bool> = if s.is_empty() {
            Box::new(|_: &&Region| true)
        } else if let Ok(id) = u32::from_str(&s) {
            Box::new(move |v: &&Region| v.id.0 == id)
        } else {
            Box::new(|v: &&Region| v.name.to_lowercase().contains(&s))
        };

        self.filter_params.region_catalog = self
            .holders
            .game_data_holder
            .region_holder
            .values()
            .filter(fun)
            .map(RegionInfo::from)
            .collect();

        self.filter_params
            .region_catalog
            .sort_by(|a, b| a.id.cmp(&b.id))
    }

    pub fn save_region_from_dlg(&mut self, id: RegionId) {
        if let CurrentEntity::Region(index) = self.edit_params.current_entity {
            let new_entity = self.edit_params.regions.opened.get(index).unwrap();

            if new_entity.inner.id() != id {
                return;
            }

            self.save_region_object_force(new_entity.inner.clone());
        }
    }

    pub(crate) fn save_region_object_force(&mut self, v: Region) {
        self.holders.game_data_holder.region_holder.insert(v.id, v);

        self.filter_regions();
    }
}

pub struct RegionInfo {
    pub(crate) id: RegionId,
    pub(crate) world_map_square: [u16; 2],
    pub(crate) name: String,
}

impl From<&Region> for RegionInfo {
    fn from(value: &Region) -> Self {
        RegionInfo {
            id: value.id,
            world_map_square: value.world_map_square,
            name: value.name.clone(),
        }
    }
}
