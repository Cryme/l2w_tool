use crate::backend::holder::FHashMap;
use crate::backend::{
    Backend, HandleAction,
};
use crate::data::QuestId;
use crate::entity::quest::Quest;
use serde::{Deserialize, Serialize};
use crate::backend::entity_editor::{CommonEditorOps, CurrentEntity, EditParams, EntityEditParams, WindowParams};

pub type QuestEditor = EntityEditParams<Quest, QuestId, QuestAction, ()>;

impl HandleAction for WindowParams<Quest, QuestId, QuestAction, ()> {
    fn handle_action(&mut self) {
        let quest = self;
        let mut action = quest.action.write().unwrap();

        match *action {
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

        *action = QuestAction::None;

        for step in &mut quest.inner.steps {
            let mut action = step.action.write().unwrap();

            match *action {
                StepAction::RemoveGoal(i) => {
                    step.inner.goals.remove(i);
                }
                StepAction::RemoveAdditionalLocation(i) => {
                    step.inner.additional_locations.remove(i);
                }
                StepAction::RemovePrevStepIndex(i) => {
                    step.inner.prev_steps.remove(i);
                }

                StepAction::None => {}
            }

            *action = StepAction::None;
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum StepAction {
    #[default]
    None,
    RemoveGoal(usize),
    RemoveAdditionalLocation(usize),
    RemovePrevStepIndex(usize),
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub enum QuestAction {
    #[default]
    None,
    RemoveStep(usize),
    RemoveStartNpcId(usize),
    RemoveReward(usize),
    RemoveQuestItem(usize),
}

impl EditParams {
    pub fn get_opened_quests_info(&self) -> Vec<(String, QuestId, bool)> {
        self.quests.get_opened_info()
    }

    pub fn open_quest(&mut self, id: QuestId, holder: &mut FHashMap<QuestId, Quest>) {
        for (i, q) in self.quests.opened.iter().enumerate() {
            if q.inner.initial_id == id {
                self.current_entity = CurrentEntity::Quest(i);

                return;
            }
        }

        if let Some(q) = holder.get(&id) {
            self.current_entity = CurrentEntity::Quest(self.quests.add(q.clone(), q.id));
        }
    }

    pub fn set_current_quest(&mut self, index: usize) {
        if index < self.quests.opened.len() {
            self.current_entity = CurrentEntity::Quest(index);
        }
    }

    pub fn create_new_quest(&mut self) {
        self.current_entity = CurrentEntity::Quest(self.quests.add_new());
    }
}

impl Backend {
    pub fn filter_quests(&mut self) {
        self.entity_catalogs.quest.filter(&self.holders.game_data_holder.quest_holder);
    }

    pub fn save_quest_from_dlg(&mut self, quest_id: QuestId) {
        if let CurrentEntity::Quest(index) = self.edit_params.current_entity {
            let new_entity = self.edit_params.quests.opened.get_mut(index).unwrap();

            if new_entity.inner.inner.id != quest_id {
                return;
            }

            new_entity.inner.initial_id = new_entity.inner.inner.id;

            let entity = new_entity.inner.inner.clone();

            self.save_quest_force(entity);
        }
    }

    pub(crate) fn save_quest_force(&mut self, mut quest: Quest) {
        if let Some(java_class) = quest.java_class {
            self.holders.server_data_holder.save_java_class(
                quest.id,
                &quest.title,
                java_class.inner,
                &self.config.server_quests_java_classes_path,
            )
        }

        quest.java_class = None;

        if let Some(vv) = self.holders.game_data_holder.quest_holder.get(&quest.id) {
            if *vv == quest{
                return;
            }
        }
        self.set_changed();

        self.holders
            .game_data_holder
            .quest_holder
            .insert(quest.id, quest);
        self.filter_quests();
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub struct QuestInfo {
    pub(crate) id: QuestId,
    pub(crate) name: String,
}

impl From<&Quest> for QuestInfo {
    fn from(value: &Quest) -> Self {
        QuestInfo {
            id: value.id,
            name: value.title.clone(),
        }
    }
}
