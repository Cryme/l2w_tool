use crate::backend::holder::FHashMap;
use crate::backend::{
    Backend, HandleAction,
};
use crate::backend::entity_editor::{CommonEditorOps, CurrentEntity, EditParams, EntityEditParams, WindowParams};
use crate::data::RegionId;
use crate::entity::region::Region;
use crate::entity::CommonEntity;

pub type RegionEditor = EntityEditParams<Region, RegionId, (), ()>;

impl HandleAction for WindowParams<Region, RegionId, (), ()> {
    fn handle_action(&mut self) {}
}

impl EditParams {
    pub fn get_opened_region_info(&self) -> Vec<(String, RegionId, bool)> {
        self.regions.get_opened_info()
    }

    pub fn open_region(&mut self, id: RegionId, holder: &mut FHashMap<RegionId, Region>) {
        for (i, q) in self.regions.opened.iter().enumerate() {
            if q.inner.initial_id == id {
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

    pub fn create_new_region(&mut self) {
        self.current_entity = CurrentEntity::Region(self.regions.add_new());
    }
}

impl Backend {
    pub fn filter_regions(&mut self) {
        self.entity_catalogs.region.filter(&self.holders.game_data_holder.region_holder);
    }

    pub fn save_region_from_dlg(&mut self, id: RegionId) {
        if let CurrentEntity::Region(index) = self.edit_params.current_entity {
            let new_entity = self.edit_params.regions.opened.get_mut(index).unwrap();

            if new_entity.inner.inner.id() != id {
                return;
            }

            new_entity.inner.initial_id = new_entity.inner.inner.id;

            let entity = new_entity.inner.inner.clone();

            self.save_region_object_force(entity);
        }
    }

    pub(crate) fn save_region_object_force(&mut self, v: Region) {
        if let Some(vv) = self.holders.game_data_holder.region_holder.get(&v.id) {
            if *vv == v{
                return;
            }
        }
        self.set_changed();

        self.holders.game_data_holder.region_holder.insert(v.id, v);

        self.filter_regions();
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
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
