use crate::backend::editor::WindowParams;
use crate::backend::entity_impl::daily_missions::DailyMissionAction;
use crate::backend::entity_impl::ensoul_option::EnsoulOptionAction;
use crate::backend::entity_impl::hunting_zone::HuntingZoneAction;
use crate::backend::entity_impl::item::armor::ArmorAction;
use crate::backend::entity_impl::item::etc_item::EtcItemAction;
use crate::backend::entity_impl::item::weapon::WeaponAction;
use crate::backend::entity_impl::item_set::ItemSetAction;
use crate::backend::entity_impl::npc::NpcAction;
use crate::backend::entity_impl::recipe::RecipeAction;
use crate::backend::entity_impl::residence::ResidenceAction;
use crate::backend::entity_impl::skill::{SkillAction, SkillEditWindowParams};
use crate::backend::gpt::GptChannelMessage;
use crate::common::{
    AnimationComboId, DailyMissionId, EnsoulOptionId, HuntingZoneId, ItemId, ItemSetId, NpcId,
    RaidInfoId, RecipeId, RegionId, ResidenceId, SkillId,
};
use crate::entity::animation_combo::AnimationCombo;
use crate::entity::daily_mission::DailyMission;
use crate::entity::ensoul_option::EnsoulOption;
use crate::entity::hunting_zone::HuntingZone;
use crate::entity::item::armor::Armor;
use crate::entity::item::etc_item::EtcItem;
use crate::entity::item::weapon::Weapon;
use crate::entity::item_set::ItemSet;
use crate::entity::npc::Npc;
use crate::entity::raid_info::RaidInfo;
use crate::entity::recipe::Recipe;
use crate::entity::region::Region;
use crate::entity::residence::Residence;
use crate::entity::skill::Skill;

pub trait HandleGptResponse {
    fn handle_gpt_message(&mut self, message: GptChannelMessage);
}

impl HandleGptResponse for WindowParams<AnimationCombo, AnimationComboId, (), ()> {
    fn handle_gpt_message(&mut self, _message: GptChannelMessage) {}
}
impl HandleGptResponse for WindowParams<RaidInfo, RaidInfoId, (), ()> {
    fn handle_gpt_message(&mut self, _message: GptChannelMessage) {}
}
impl HandleGptResponse for WindowParams<Recipe, RecipeId, RecipeAction, ()> {
    fn handle_gpt_message(&mut self, _message: GptChannelMessage) {}
}
impl HandleGptResponse for WindowParams<Region, RegionId, (), ()> {
    fn handle_gpt_message(&mut self, _message: GptChannelMessage) {}
}
impl HandleGptResponse for WindowParams<Residence, ResidenceId, ResidenceAction, ()> {
    fn handle_gpt_message(&mut self, _message: GptChannelMessage) {}
}
impl HandleGptResponse for WindowParams<Skill, SkillId, SkillAction, SkillEditWindowParams> {
    fn handle_gpt_message(&mut self, _message: GptChannelMessage) {}
}
impl HandleGptResponse for WindowParams<DailyMission, DailyMissionId, DailyMissionAction, ()> {
    fn handle_gpt_message(&mut self, _message: GptChannelMessage) {}
}
impl HandleGptResponse for WindowParams<EnsoulOption, EnsoulOptionId, EnsoulOptionAction, ()> {
    fn handle_gpt_message(&mut self, _message: GptChannelMessage) {}
}
impl HandleGptResponse for WindowParams<HuntingZone, HuntingZoneId, HuntingZoneAction, ()> {
    fn handle_gpt_message(&mut self, _message: GptChannelMessage) {}
}
impl HandleGptResponse for WindowParams<Armor, ItemId, ArmorAction, ()> {
    fn handle_gpt_message(&mut self, _message: GptChannelMessage) {}
}
impl HandleGptResponse for WindowParams<EtcItem, ItemId, EtcItemAction, ()> {
    fn handle_gpt_message(&mut self, _message: GptChannelMessage) {}
}
impl HandleGptResponse for WindowParams<Weapon, ItemId, WeaponAction, ()> {
    fn handle_gpt_message(&mut self, _message: GptChannelMessage) {}
}
impl HandleGptResponse for WindowParams<ItemSet, ItemSetId, ItemSetAction, ()> {
    fn handle_gpt_message(&mut self, _message: GptChannelMessage) {}
}
impl HandleGptResponse for WindowParams<Npc, NpcId, NpcAction, ()> {
    fn handle_gpt_message(&mut self, _message: GptChannelMessage) {}
}
