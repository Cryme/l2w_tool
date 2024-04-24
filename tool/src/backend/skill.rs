use crate::backend::{Backend, CurrentOpenedEntity, EditParams};
use crate::data::SkillId;
use crate::entity::skill::Skill;
use crate::holders::{FHashMap, SkillInfo};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Clone, Default)]
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

#[derive(Serialize, Deserialize, Clone, Default)]
pub enum SkillUceConditionAction {
    #[default]
    None,
    DeleteWeapon(usize),
    DeleteEffectOnCaster(usize),
    DeleteEffectOnTarget(usize),
}

#[derive(Serialize, Deserialize, Default)]
pub enum SkillEnchantAction {
    #[default]
    None,
}

#[derive(Serialize, Deserialize, Default)]
pub struct SkillEditWindowParams {
    pub current_level_index: usize,
}

#[derive(Serialize, Deserialize, Default, Clone)]
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
