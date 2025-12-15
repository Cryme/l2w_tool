use crate::entity::animation_combo::AnimationCombo;
use crate::entity::daily_mission::DailyMission;
use crate::entity::ensoul_option::EnsoulOption;
use crate::entity::hunting_zone::HuntingZone;
use crate::entity::item::armor::Armor;
use crate::entity::item::etc_item::EtcItem;
use crate::entity::item::weapon::Weapon;
use crate::entity::item_set::ItemSet;
use crate::entity::npc::Npc;
use crate::entity::quest::Quest;
use crate::entity::raid_info::RaidInfo;
use crate::entity::recipe::Recipe;
use crate::entity::region::Region;
use crate::entity::residence::Residence;
use crate::entity::skill::Skill;

pub trait EntityIdOps {
    fn set_id_from_u32(&mut self, val: u32);
    fn get_id_as_u32(&self) -> u32;
}

impl EntityIdOps for AnimationCombo {
    fn set_id_from_u32(&mut self, val: u32) {
        self.id.0 = val;
    }

    fn get_id_as_u32(&self) -> u32 {
        self.id.0
    }
}
impl EntityIdOps for Armor {
    fn set_id_from_u32(&mut self, val: u32) {
        self.base_info.id.0 = val;
    }

    fn get_id_as_u32(&self) -> u32 {
        self.base_info.id.0
    }
}
impl EntityIdOps for Weapon {
    fn set_id_from_u32(&mut self, val: u32) {
        self.base_info.id.0 = val;
    }

    fn get_id_as_u32(&self) -> u32 {
        self.base_info.id.0
    }
}
impl EntityIdOps for EtcItem {
    fn set_id_from_u32(&mut self, val: u32) {
        self.base_info.id.0 = val;
    }

    fn get_id_as_u32(&self) -> u32 {
        self.base_info.id.0
    }
}
impl EntityIdOps for DailyMission {
    fn set_id_from_u32(&mut self, val: u32) {
        self.id.0 = val;
    }

    fn get_id_as_u32(&self) -> u32 {
        self.id.0
    }
}
impl EntityIdOps for EnsoulOption {
    fn set_id_from_u32(&mut self, val: u32) {
        self.id.0 = val;
    }

    fn get_id_as_u32(&self) -> u32 {
        self.id.0
    }
}
impl EntityIdOps for HuntingZone {
    fn set_id_from_u32(&mut self, val: u32) {
        self.id.0 = val;
    }

    fn get_id_as_u32(&self) -> u32 {
        self.id.0
    }
}
impl EntityIdOps for ItemSet {
    fn set_id_from_u32(&mut self, val: u32) {
        self.id.0 = val;
    }

    fn get_id_as_u32(&self) -> u32 {
        self.id.0
    }
}
impl EntityIdOps for Npc {
    fn set_id_from_u32(&mut self, val: u32) {
        self.id.0 = val;
    }

    fn get_id_as_u32(&self) -> u32 {
        self.id.0
    }
}
impl EntityIdOps for Quest {
    fn set_id_from_u32(&mut self, val: u32) {
        self.id.0 = val;
    }

    fn get_id_as_u32(&self) -> u32 {
        self.id.0
    }
}
impl EntityIdOps for RaidInfo {
    fn set_id_from_u32(&mut self, val: u32) {
        self.id.0 = val;
    }

    fn get_id_as_u32(&self) -> u32 {
        self.id.0
    }
}
impl EntityIdOps for Recipe {
    fn set_id_from_u32(&mut self, val: u32) {
        self.id.0 = val;
    }

    fn get_id_as_u32(&self) -> u32 {
        self.id.0
    }
}
impl EntityIdOps for Region {
    fn set_id_from_u32(&mut self, val: u32) {
        self.id.0 = val;
    }

    fn get_id_as_u32(&self) -> u32 {
        self.id.0
    }
}
impl EntityIdOps for Residence {
    fn set_id_from_u32(&mut self, val: u32) {
        self.id.0 = val;
    }

    fn get_id_as_u32(&self) -> u32 {
        self.id.0
    }
}
impl EntityIdOps for Skill {
    fn set_id_from_u32(&mut self, val: u32) {
        self.id.0 = val;
    }

    fn get_id_as_u32(&self) -> u32 {
        self.id.0
    }
}
