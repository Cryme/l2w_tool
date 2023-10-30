use crate::data::QuestId;
use crate::holders::{ChroniclesProtocol, GameDataHolder, load_holder, QuestInfo};
use std::collections::HashMap;
use std::str::FromStr;
use crate::entity::quest::Quest;


#[derive(Debug, Copy, Clone)]
pub enum StepAction {
    None,
    RemoveGoal(usize),
    RemoveAdditionalLocation(usize),
    RemovePrevStepIndex(usize),
}

pub enum QuestAction {
    None,
    RemoveStep(usize),
    RemoveStartNpcId(usize),
    RemoveReward(usize),
    RemoveQuestItem(usize),
}


#[derive(Debug, Clone)]
pub struct WindowParams<T, A> {
    pub(crate) inner: T,
    pub(crate) opened: bool,
    pub(crate) action: A,
}

pub struct Backend {
    pub quest_edit_params: QuestEditParams,
    pub holder: GameDataHolder,
    pub filter_params: FilterParams,
}

pub struct FilterParams {
    pub quest_filter_string: String,
    pub quest_catalog: Vec<QuestInfo>,
}

pub struct QuestEditParams {
    next_quest_id: u32,
    pub current_quest: Option<WindowParams<Quest, QuestAction>>,
}

impl QuestEditParams {
    pub fn set_current_quest(&mut self, id: QuestId, holder: &mut HashMap<QuestId, Quest>) {
        if let Some(q) = holder.get(&id) {
            self.current_quest = Some(WindowParams {
                inner: q.clone_and_replace_escaped(),
                opened: false,
                action: QuestAction::None,
            })
        }
    }


    pub fn create_new_quest(&mut self) {
        self.current_quest = Some(WindowParams {
            inner: Quest::new(self.next_quest_id),
            opened: false,
            action: QuestAction::None,
        });

        self.next_quest_id += 1;
    }

}

impl Backend {
    pub fn init() -> Self {
        let holder = load_holder("/home/cryme/RustroverProjects/l2w_tool/game_system", ChroniclesProtocol::GrandCrusade110).unwrap();
        let mut r = Self {
            quest_edit_params: QuestEditParams {
                next_quest_id: 0,
                current_quest: None,
            },
            holder,
            filter_params: FilterParams {
                quest_filter_string: "".to_string(),
                quest_catalog: vec![],
            },
        };

        r.filter_quests();

        r.quest_edit_params.next_quest_id = if let Some(last) = r.filter_params.quest_catalog.last() {last.id.0 + 1} else { 0 };

        r
    }

    pub fn filter_quests(&mut self) {
        let s = self.filter_params.quest_filter_string.clone();
        let fun: Box<dyn Fn(&&Quest) -> bool> = if s.is_empty() {
            Box::new(|_: &&Quest| true)
        } else {
            if let Ok(id) = u32::from_str(&s) {
                Box::new(move |v: &&Quest| v.id == QuestId(id))
            } else {
                Box::new(move |v: &&Quest| v.title.contains(&s))
            }
        };

        self.filter_params.quest_catalog = self.holder.quest_holder.values().filter(fun).map(|v| v.into()).collect();
        self.filter_params.quest_catalog.sort_by(|a, b| a.id.cmp(&b.id))
    }

    pub fn remove_deleted(&mut self) {
        if let Some(quest) = &mut self.quest_edit_params.current_quest {
            match quest.action {
                QuestAction::RemoveStep(i) => {
                    quest.inner.steps.remove(i);
                }
                QuestAction::RemoveStartNpcId(i) => {
                    quest.inner.start_npc_ids.remove(i);
                }
                QuestAction::RemoveReward(i) => {
                    quest.inner.rewards.remove(i);
                }
                QuestAction::RemoveQuestItem(i) => {
                    quest.inner.quest_items.remove(i);
                }

                QuestAction::None => {}
            }

            quest.action = QuestAction::None;

            for step in &mut quest.inner.steps {
                match step.action {
                    StepAction::RemoveGoal(i) => {
                        step.inner.goals.remove(i);
                    }
                    StepAction::RemoveAdditionalLocation(i) => {
                        step.inner.additional_locations.remove(i);
                    }
                    StepAction::RemovePrevStepIndex(i) => {
                        step.inner.prev_step_indexes.remove(i);
                    }

                    StepAction::None => {}
                }

                step.action = StepAction::None;
            }
        }
    }
}
