use crate::holders::{load_holder, ChroniclesProtocol, GameDataHolder};
use crate::quest::Quest;

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

pub struct WindowParams<T, A> {
    pub(crate) inner: T,
    pub(crate) opened: bool,
    pub(crate) action: A,
}

pub struct Backend {
    next_quest_id: u32,
    pub current_quest: Option<WindowParams<Quest, QuestAction>>,
    pub holder: GameDataHolder,
}

impl Backend {
    pub fn init() -> Self {
        Self {
            next_quest_id: 0,
            current_quest: None,
            holder: load_holder("/home/cryme/RustroverProjects/l2w_tool/game_system", ChroniclesProtocol::GrandCrusade110).unwrap(),
            // holder: load_holder("F:/RustroverProjects/l2_quest_tool/game_system", ChroniclesProtocol::GrandCrusade110).unwrap(),
        }
    }

    pub fn remove_deleted(&mut self) {
        if let Some(quest) = &mut self.current_quest {
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

    pub fn create_new_quest(&mut self) {
        self.current_quest = Some(WindowParams {
            inner: Quest::new(self.next_quest_id),
            opened: false,
            action: QuestAction::None,
        });

        self.next_quest_id += 1;
    }
}
