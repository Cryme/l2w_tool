use crate::backend::editor::entity::{CommonEditorOps, EntityEditParams};
use crate::backend::editor::{CurrentEntity, EditParamsCommonOps, Editors, WindowParams};
use crate::backend::entity_catalog::EntityInfo;
use crate::backend::holder::{FHashMap, HolderMapOps};
use crate::backend::{Backend, HandleAction};
use crate::common::RegionId;
use crate::entity::CommonEntity;
use crate::entity::region::Region;

pub type RegionEditor = EntityEditParams<Region, RegionId, (), ()>;

impl HandleAction for WindowParams<Region, RegionId, (), ()> {
    fn handle_action(&mut self) {}
}

impl Editors {
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
            self.current_entity = CurrentEntity::Region(self.regions.add(q.clone(), q.id(), false));
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
        self.entity_catalogs.region.filter(
            &self.holders.game_data_holder.region_holder,
            self.entity_catalogs.filter_mode,
        );
    }

    pub fn save_region_from_dlg(&mut self, id: RegionId) {
        if let CurrentEntity::Region(index) = self.editors.current_entity {
            let new_entity = self.editors.regions.opened.get_mut(index).unwrap();

            if new_entity.inner.inner.id() != id {
                return;
            }

            new_entity.inner.initial_id = new_entity.inner.inner.id;

            let entity = new_entity.inner.inner.clone();

            new_entity.on_save();

            self.save_region_object_force(entity);
        }
    }

    pub(crate) fn save_region_object_force(&mut self, mut v: Region) {
        if let Some(vv) = self.holders.game_data_holder.region_holder.get(&v.id) {
            if *vv == v {
                return;
            }
        }
        v._changed = true;

        self.holders.game_data_holder.region_holder.insert(v.id, v);

        self.filter_regions();
        self.check_for_unwrote_changed();
    }
}

impl From<&Region> for EntityInfo<Region, RegionId> {
    fn from(value: &Region) -> Self {
        EntityInfo::new(
            &format!(
                "ID: {}\n[{}_{}] {}",
                value.id.0, value.world_map_square[0], value.world_map_square[1], value.name
            ),
            value,
        )
    }
}
