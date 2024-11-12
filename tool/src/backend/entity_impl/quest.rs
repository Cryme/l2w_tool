use crate::backend::editor::entity::{CommonEditorOps, EntityEditParams};
use crate::backend::editor::{CurrentEntity, EditParamsCommonOps, Editors, WindowParams};
use crate::backend::entity_catalog::EntityInfo;
use crate::backend::holder::{FHashMap, HolderMapOps};
use crate::backend::{Backend, HandleAction};
use crate::common::QuestId;
use crate::entity::quest::Quest;
use serde::{Deserialize, Serialize};

pub type QuestEditor = EntityEditParams<
    Quest,
    QuestId,
    QuestAction,
    crate::frontend::node_editor::NodeEditorConnectionInfo,
>;

impl HandleAction
    for WindowParams<
        Quest,
        QuestId,
        QuestAction,
        crate::frontend::node_editor::NodeEditorConnectionInfo,
    >
{
    fn handle_action(&mut self) {
        let quest = self;
        let mut action = quest.action.write().unwrap();

        match *action {
            QuestAction::RemoveStep(i) => {
                let mut for_remove = vec![];
                for step in &mut quest.inner.steps {
                    for_remove.clear();

                    for (i, s) in step.prev_steps.iter_mut().enumerate() {
                        if *s == i {
                            for_remove.push(i);
                        } else if *s > i {
                            *s -= 1;
                        }
                    }

                    for i in for_remove.iter().rev() {
                        step.prev_steps.remove(*i);
                    }
                }

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
            QuestAction::RemoveStepGoal {
                step_index,
                goal_index,
            } => {
                quest.inner.steps[step_index].goals.remove(goal_index);
            }
            QuestAction::RemoveStepAdditionalLocation {
                step_index,
                location_index,
            } => {
                quest.inner.steps[step_index]
                    .additional_locations
                    .remove(location_index);
            }

            QuestAction::None => {}
        }

        *action = QuestAction::None;
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub enum QuestAction {
    #[default]
    None,
    RemoveStep(usize),
    RemoveStepGoal {
        step_index: usize,
        goal_index: usize,
    },
    RemoveStepAdditionalLocation {
        step_index: usize,
        location_index: usize,
    },
    RemoveStartNpcId(usize),
    RemoveReward(usize),
    RemoveQuestItem(usize),
}

impl Editors {
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

        if let Some(q) = holder.get_mut(&id) {
            q.sort_steps();

            self.current_entity = CurrentEntity::Quest(self.quests.add(q.clone(), q.id, false));
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
        self.entity_catalogs.quest.filter(
            &self.holders.game_data_holder.quest_holder,
            self.entity_catalogs.filter_mode,
        );
    }

    pub fn save_quest_from_dlg(&mut self, quest_id: QuestId) {
        if let CurrentEntity::Quest(index) = self.editors.current_entity {
            let new_entity = self.editors.quests.opened.get_mut(index).unwrap();

            if new_entity.inner.inner.id != quest_id {
                return;
            }

            new_entity.inner.initial_id = new_entity.inner.inner.id;

            let entity = new_entity.inner.inner.clone();

            new_entity.on_save();

            self.save_quest_force(entity);
        }
    }

    pub(crate) fn save_quest_force(&mut self, mut v: Quest) {
        if let Some(java_class) = v.java_class {
            self.holders.server_data_holder.save_java_class(
                v.id,
                &v.title,
                java_class.inner,
                &self.config.server_quests_java_classes_path,
            )
        }

        v.java_class = None;
        v._changed = true;

        if let Some(vv) = self.holders.game_data_holder.quest_holder.get(&v.id) {
            if *vv == v {
                return;
            }
        }

        self.holders.game_data_holder.quest_holder.insert(v.id, v);

        self.filter_quests();
        self.check_for_unwrote_changed();
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub struct QuestInfo {
    pub(crate) id: QuestId,
    pub(crate) name: String,
}

impl From<&Quest> for EntityInfo<Quest, QuestId> {
    fn from(value: &Quest) -> Self {
        EntityInfo::new(&format!("ID: {}\n{}", value.id.0, value.title), value)
    }
}
