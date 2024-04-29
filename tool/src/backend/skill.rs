use crate::backend::{
    Backend, CommonEditorOps, CurrentOpenedEntity, EditParams, EntityEditParams, HandleAction,
    WindowParams,
};
use crate::data::SkillId;
use crate::entity::skill::{EnchantInfo, EnchantLevelInfo, Skill, SkillLevelInfo};
use crate::holder::FHashMap;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::RwLock;

pub type SkillEditor = EntityEditParams<Skill, SkillId, SkillAction, SkillEditWindowParams>;

impl HandleAction for SkillEditor {
    fn handle_action(&mut self, index: usize) {
        let skill = &mut self.opened[index];

        let mut action = skill.action.write().unwrap();

        match *action {
            SkillAction::DeleteLevel => {
                skill
                    .inner
                    .skill_levels
                    .remove(skill.params.current_level_index);

                for level in &mut skill.inner.skill_levels[skill.params.current_level_index..] {
                    level.level -= 1
                }

                skill.params.current_level_index = skill
                    .params
                    .current_level_index
                    .min(skill.inner.skill_levels.len() - 1)
                    .max(0)
            }
            SkillAction::AddLevel => {
                let mut new_level_level = 1;
                let mut proto_index = 0;

                for (i, level) in skill.inner.skill_levels.iter().enumerate() {
                    proto_index = i;

                    if level.level > new_level_level {
                        break;
                    }

                    new_level_level += 1;
                }

                let mut new_level = if skill.inner.skill_levels.is_empty() {
                    SkillLevelInfo::default()
                } else {
                    skill.inner.skill_levels[proto_index].clone()
                };

                new_level.level = new_level_level;

                if proto_index != skill.inner.skill_levels.len() - 1 {
                    skill.inner.skill_levels.insert(proto_index, new_level);
                    skill.params.current_level_index = proto_index;
                } else {
                    skill.params.current_level_index = skill.inner.skill_levels.len();
                    skill.inner.skill_levels.push(new_level);
                }
            }
            SkillAction::AddEnchant => {
                let curr_level = &mut skill.inner.skill_levels[skill.params.current_level_index];

                curr_level.available_enchants.push(
                    if let Some(v) = curr_level.available_enchants.last() {
                        let mut r = v.inner.clone();

                        r.enchant_type = v.inner.enchant_type + 1;

                        WindowParams {
                            inner: r,
                            opened: false,
                            original_id: (),
                            action: RwLock::new(SkillEnchantAction::None),
                            params: SkillEnchantEditWindowParams {
                                current_level_index: v.inner.enchant_levels.len() - 1,
                            },
                        }
                    } else {
                        WindowParams {
                            inner: EnchantInfo::default(),
                            opened: false,
                            original_id: (),
                            action: RwLock::new(SkillEnchantAction::None),
                            params: SkillEnchantEditWindowParams {
                                current_level_index: 0,
                            },
                        }
                    },
                )
            }
            SkillAction::DeleteEnchant(index) => {
                let curr_level = &mut skill.inner.skill_levels[skill.params.current_level_index];
                curr_level.available_enchants.remove(index);
            }
            SkillAction::AddEnchantLevel(index) => {
                let curr_enchant = &mut skill.inner.skill_levels[skill.params.current_level_index]
                    .available_enchants[index];
                let mut new_level_level = 1;
                let mut proto_index = 0;

                for (i, level) in curr_enchant.inner.enchant_levels.iter().enumerate() {
                    proto_index = i;

                    if level.level > new_level_level {
                        break;
                    }

                    new_level_level += 1;
                }

                let mut new_level = if curr_enchant.inner.enchant_levels.is_empty() {
                    EnchantLevelInfo::default()
                } else {
                    curr_enchant.inner.enchant_levels[proto_index].clone()
                };

                new_level.level = new_level_level;

                if proto_index != curr_enchant.inner.enchant_levels.len() - 1 {
                    curr_enchant
                        .inner
                        .enchant_levels
                        .insert(proto_index, new_level);
                    curr_enchant.params.current_level_index = proto_index;
                } else {
                    curr_enchant.params.current_level_index =
                        curr_enchant.inner.enchant_levels.len();
                    curr_enchant.inner.enchant_levels.push(new_level);
                }
            }
            SkillAction::DeleteEnchantLevel(index) => {
                let curr_enchant = &mut skill.inner.skill_levels[skill.params.current_level_index]
                    .available_enchants[index];
                curr_enchant
                    .inner
                    .enchant_levels
                    .remove(curr_enchant.params.current_level_index);
                curr_enchant.params.current_level_index = curr_enchant
                    .params
                    .current_level_index
                    .min(curr_enchant.inner.enchant_levels.len() - 1)
                    .max(0)
            }

            SkillAction::None => {}
        }

        *action = SkillAction::None;

        if let Some(cond) = &mut skill.inner.use_condition {
            let mut action = cond.action.write().unwrap();

            match *action {
                SkillUceConditionAction::DeleteWeapon(i) => {
                    cond.inner.weapon_types.remove(i);
                }
                SkillUceConditionAction::DeleteEffectOnCaster(i) => {
                    cond.inner.caster_prior_skill.remove(i);
                }
                SkillUceConditionAction::DeleteEffectOnTarget(i) => {
                    cond.inner.target_prior_skill.remove(i);
                }

                SkillUceConditionAction::None => {}
            }

            *action = SkillUceConditionAction::None;
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub enum SkillAction {
    #[default]
    None,
    DeleteLevel,
    AddLevel,
    AddEnchant,
    DeleteEnchant(usize),
    AddEnchantLevel(usize),
    DeleteEnchantLevel(usize),
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub enum SkillUceConditionAction {
    #[default]
    None,
    DeleteWeapon(usize),
    DeleteEffectOnCaster(usize),
    DeleteEffectOnTarget(usize),
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub enum SkillEnchantAction {
    #[default]
    None,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct SkillEditWindowParams {
    pub current_level_index: usize,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct SkillEnchantEditWindowParams {
    pub current_level_index: usize,
}

impl EditParams {
    pub fn get_opened_skills_info(&self) -> Vec<(String, SkillId)> {
        self.skills.get_opened_info()
    }

    pub fn open_skill(&mut self, id: SkillId, holder: &mut FHashMap<SkillId, Skill>) {
        for (i, q) in self.skills.opened.iter().enumerate() {
            if q.original_id == id {
                self.current_opened_entity = CurrentOpenedEntity::Skill(i);

                return;
            }
        }

        if let Some(q) = holder.get(&id) {
            self.current_opened_entity =
                CurrentOpenedEntity::Skill(self.skills.add(q.clone(), q.id));
        }
    }

    pub fn set_current_skill(&mut self, index: usize) {
        if index < self.skills.opened.len() {
            self.current_opened_entity = CurrentOpenedEntity::Skill(index);
        }
    }

    pub fn close_skill(&mut self, index: usize) {
        self.skills.opened.remove(index);

        if let CurrentOpenedEntity::Skill(curr_index) = self.current_opened_entity {
            if self.skills.opened.is_empty() {
                self.find_opened_entity();
            } else if curr_index >= index {
                self.current_opened_entity = CurrentOpenedEntity::Skill(curr_index.max(1) - 1)
            }
        }
    }

    pub fn create_new_skill(&mut self) {
        self.current_opened_entity = CurrentOpenedEntity::Skill(self.skills.add_new());
    }
}

impl Backend {
    pub fn filter_skills(&mut self) {
        let mut s = self.filter_params.skill_filter_string.clone();

        let fun: Box<dyn Fn(&&Skill) -> bool> = if s.is_empty() {
            Box::new(|_: &&Skill| true)
        } else if let Some(_stripped) = s.strip_prefix('~') {
            Box::new(move |_: &&Skill| false)
        } else if let Some(stripped) = s.strip_prefix("id:") {
            if let Ok(id) = u32::from_str(stripped) {
                Box::new(move |v: &&Skill| v.id == SkillId(id))
            } else {
                Box::new(|_: &&Skill| false)
            }
        } else {
            let invert = s.starts_with('!');

            if invert {
                s = s[1..].to_string();
            }

            Box::new(move |v: &&Skill| {
                let r = v.name.contains(&s)
                    || v.description.contains(&s)
                    || v.animations[0].to_string().contains(&s)
                    || v.icon.contains(&s)
                    || v.icon_panel.contains(&s);

                if invert {
                    !r
                } else {
                    r
                }
            })
        };

        self.filter_params.skill_catalog = self
            .holders
            .game_data_holder
            .skill_holder
            .values()
            .filter(fun)
            .map(SkillInfo::from)
            .collect();

        self.filter_params
            .skill_catalog
            .sort_by(|a, b| a.id.cmp(&b.id))
    }

    pub fn save_skill_from_dlg(&mut self, skill_id: SkillId) {
        if let CurrentOpenedEntity::Skill(index) = self.edit_params.current_opened_entity {
            let new_skill = self.edit_params.skills.opened.get(index).unwrap();

            if new_skill.inner.id != skill_id {
                return;
            }

            self.save_skill_force(new_skill.inner.clone());
        }
    }

    pub(crate) fn save_skill_force(&mut self, skill: Skill) {
        self.holders
            .game_data_holder
            .skill_holder
            .insert(skill.id, skill);
        self.filter_skills();
    }
}

pub struct SkillInfo {
    pub(crate) id: SkillId,
    pub(crate) name: String,
}

impl From<&Skill> for SkillInfo {
    fn from(value: &Skill) -> Self {
        SkillInfo {
            id: value.id,
            name: value.name.clone(),
        }
    }
}
