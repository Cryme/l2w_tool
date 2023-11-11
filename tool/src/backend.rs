use crate::data::{QuestId, SkillId};
use crate::entity::quest::Quest;
use crate::entity::skill::Skill;
use crate::holders::{
    get_loader_from_holder, load_game_data_holder, ChroniclesProtocol, GameDataHolder, Loader,
    QuestInfo,
};
use crate::server_side::ServerDataHolder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{Duration, SystemTime};

const AUTO_SAVE_FILE_NAME: &str = "./auto.save";
const AUTO_SAVE_INTERVAL: Duration = Duration::from_secs(30);
const CONFIG_FILE_NAME: &str = "./config.ron";

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum StepAction {
    None,
    RemoveGoal(usize),
    RemoveAdditionalLocation(usize),
    RemovePrevStepIndex(usize),
}

#[derive(Serialize, Deserialize)]
pub enum QuestAction {
    None,
    RemoveStep(usize),
    RemoveStartNpcId(usize),
    RemoveReward(usize),
    RemoveQuestItem(usize),
}

#[derive(Serialize, Deserialize)]
pub enum SkillAction {
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WindowParams<T, I, A> {
    pub(crate) inner: T,
    pub(crate) opened: bool,
    pub(crate) original_id: I,
    pub(crate) action: A,
}

pub struct FilterParams {
    pub quest_filter_string: String,
    pub quest_catalog: Vec<QuestInfo>,
}

#[derive(Serialize, Deserialize, Default, Eq, PartialEq)]
pub enum CurrentOpenedEntity {
    #[default]
    None,
    Quest(usize),
    Skill(usize),
}

impl CurrentOpenedEntity {
    pub fn is_some(&self) -> bool {
        *self != CurrentOpenedEntity::None
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct SkillEditParams {
    next_quest_id: u32,
    pub opened: Vec<WindowParams<Skill, SkillId, SkillAction>>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct QuestEditParams {
    next_quest_id: u32,
    pub opened: Vec<WindowParams<Quest, QuestId, QuestAction>>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct EditParams {
    pub quest_edit_params: QuestEditParams,
    pub skills: SkillEditParams,
    pub current_opened_entity: CurrentOpenedEntity,
}

impl EditParams {
    pub fn get_opened_quests_info(&self) -> Vec<(String, QuestId)> {
        self.quest_edit_params.get_opened_quests_info()
    }

    pub fn open_quest(&mut self, id: QuestId, holder: &mut HashMap<QuestId, Quest>) {
        for (i, q) in self.quest_edit_params.opened.iter().enumerate() {
            if q.original_id == id {
                self.current_opened_entity = CurrentOpenedEntity::Quest(i);

                return;
            }
        }

        if let Some(q) = holder.get(&id) {
            self.current_opened_entity =
                CurrentOpenedEntity::Quest(self.quest_edit_params.add_quest(q.clone(), q.id));
        }
    }

    pub fn set_current_quest(&mut self, index: usize) {
        if index < self.quest_edit_params.opened.len() {
            self.current_opened_entity = CurrentOpenedEntity::Quest(index);
        }
    }

    pub fn close_quest(&mut self, index: usize) {
        self.quest_edit_params.opened.remove(index);

        if let CurrentOpenedEntity::Quest(curr_index) = self.current_opened_entity {
            if self.quest_edit_params.opened.is_empty() {
                self.find_opened_entity();
            } else if curr_index >= index {
                self.current_opened_entity = CurrentOpenedEntity::Quest(curr_index.max(1) - 1)
            }
        }
    }

    fn find_opened_entity(&mut self) {
        if !self.quest_edit_params.opened.is_empty() {
            self.current_opened_entity =
                CurrentOpenedEntity::Quest(self.quest_edit_params.opened.len() - 1);
        // } else if self.skill_edit_params.opened.len() > 0  {
        } else {
            self.current_opened_entity = CurrentOpenedEntity::None;
        }
    }

    pub fn create_new_quest(&mut self) {
        self.quest_edit_params.add_new_quest();
        self.current_opened_entity =
            CurrentOpenedEntity::Quest(self.quest_edit_params.opened.len() - 1);
    }
}

impl QuestEditParams {
    fn get_opened_quests_info(&self) -> Vec<(String, QuestId)> {
        self.opened
            .iter()
            .map(|v| (v.inner.title.clone(), v.inner.id))
            .collect()
    }

    fn add_quest(&mut self, quest: Quest, original_id: QuestId) -> usize {
        self.opened.push(WindowParams {
            inner: quest,
            original_id,
            opened: false,
            action: QuestAction::None,
        });

        self.opened.len() - 1
    }

    fn add_new_quest(&mut self) -> usize {
        self.add_quest(Quest::new(self.next_quest_id), QuestId(self.next_quest_id));

        self.next_quest_id += 1;

        self.opened.len() - 1
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub system_folder_path: Option<String>,
    pub quest_java_classes_path: Option<String>,
}

impl Config {
    fn dump(&self) {
        let mut f = File::create(Path::new(CONFIG_FILE_NAME)).unwrap();
        f.write_all(
            (ron::ser::to_string_pretty::<Config>(self, ron::ser::PrettyConfig::default())
                .unwrap())
            .as_ref(),
        )
        .unwrap();
    }
}

pub struct Holders {
    pub game_data_holder: GameDataHolder,
    pub server_data_holder: ServerDataHolder,
}

impl Holders {
    pub fn set_java_class(&mut self, quest: &mut Quest) {
        if let Some(v) = self.server_data_holder.quest_java_classes.get(&quest.id) {
            let mut class = "".to_string();

            File::open(v.path())
                .unwrap()
                .read_to_string(&mut class)
                .unwrap();

            quest.java_class = Some(WindowParams {
                inner: class,
                original_id: (),
                opened: false,
                action: (),
            });
        } else {
            quest.java_class = Some(WindowParams {
                inner: self
                    .server_data_holder
                    .generate_java_template(quest, &self.game_data_holder),
                original_id: (),
                opened: false,
                action: (),
            });
        }
    }
}

struct Tasks {
    last_auto_save: SystemTime,
}

impl Tasks {
    fn init() -> Self {
        Self {
            last_auto_save: SystemTime::now(),
        }
    }
}

pub struct Backend {
    pub config: Config,
    pub holders: Holders,
    pub filter_params: FilterParams,
    pub dialog: Dialog,
    pub dialog_showing: bool,
    pub edit_params: EditParams,

    tasks: Tasks,
}

#[derive(Eq, PartialEq)]
pub enum DialogAnswer {
    Confirm,
    Abort,
}

pub enum Dialog {
    None,
    ConfirmQuestSave { message: String, quest_id: QuestId },
    ShowWarning(String),
}

impl Backend {
    pub(crate) fn save_to_dat(&self) {
        let loader = get_loader_from_holder(&self.holders.game_data_holder);

        loader.serialize_to_binary().unwrap();
    }

    fn load_config() -> Config {
        let config_path = Path::new(CONFIG_FILE_NAME);
        if let Ok(mut f) = File::open(config_path) {
            let mut d = "".to_string();
            if f.read_to_string(&mut d).is_ok() {
                if let Ok(mut c) = ron::from_str(&d) {
                    ServerDataHolder::validate_paths(&mut c);
                    GameDataHolder::validate_paths(&mut c);

                    c.dump();

                    return c;
                }
            }
        }
        let c = Config::default();
        c.dump();

        c
    }

    pub fn init() -> Self {
        let config = Self::load_config();

        let game_data_holder = if let Some(path) = &config.system_folder_path {
            load_game_data_holder(path, ChroniclesProtocol::GrandCrusade110).unwrap()
        } else {
            GameDataHolder::default()
        };

        let server_data_holder = if let Some(path) = &config.quest_java_classes_path {
            ServerDataHolder::load(path)
        } else {
            ServerDataHolder::default()
        };

        let edit_params = if let Ok(f) = File::open(AUTO_SAVE_FILE_NAME) {
            if let Ok(d) = bincode::deserialize_from::<File, EditParams>(f) {
                d
            } else {
                EditParams::default()
            }
        } else {
            EditParams::default()
        };

        let mut r = Self {
            config,

            holders: Holders {
                game_data_holder,
                server_data_holder,
            },
            filter_params: FilterParams {
                quest_filter_string: "".to_string(),
                quest_catalog: vec![],
            },
            dialog: Dialog::None,
            dialog_showing: false,

            tasks: Tasks::init(),
            edit_params,
        };

        r.update_last_id();

        r
    }

    fn update_last_id(&mut self) {
        self.filter_params.quest_filter_string = "".to_string();
        self.filter_quests();

        self.edit_params.quest_edit_params.next_quest_id =
            if let Some(last) = self.filter_params.quest_catalog.last() {
                last.id.0 + 1
            } else {
                0
            };
    }

    pub fn filter_quests(&mut self) {
        let s = self.filter_params.quest_filter_string.clone();
        let fun: Box<dyn Fn(&&Quest) -> bool> = if s.is_empty() {
            Box::new(|_: &&Quest| true)
        } else if let Ok(id) = u32::from_str(&s) {
            Box::new(move |v: &&Quest| v.id == QuestId(id))
        } else {
            Box::new(move |v: &&Quest| v.title.contains(&s))
        };

        self.filter_params.quest_catalog = self
            .holders
            .game_data_holder
            .quest_holder
            .values()
            .filter(fun)
            .map(QuestInfo::from)
            .collect();
        self.filter_params
            .quest_catalog
            .sort_by(|a, b| a.id.cmp(&b.id))
    }

    pub fn auto_save(&mut self, force: bool) {
        if !force
            && SystemTime::now()
                .duration_since(self.tasks.last_auto_save)
                .unwrap()
                < AUTO_SAVE_INTERVAL
        {
            return;
        }

        if let Ok(mut out) = File::create(AUTO_SAVE_FILE_NAME) {
            out.write_all(&bincode::serialize(&self.edit_params).unwrap())
                .unwrap();
        }

        self.tasks.last_auto_save = SystemTime::now();
    }

    fn remove_deleted(&mut self) {
        match self.edit_params.current_opened_entity {
            CurrentOpenedEntity::Quest(index) => {
                let quest = &mut self.edit_params.quest_edit_params.opened[index];
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
                            step.inner.prev_steps.remove(i);
                        }

                        StepAction::None => {}
                    }

                    step.action = StepAction::None;
                }
            }
            CurrentOpenedEntity::Skill(_) => {}
            CurrentOpenedEntity::None => {}
        }
    }

    pub fn on_update(&mut self) {
        self.remove_deleted();
        self.auto_save(false);
    }

    pub fn update_system_path(&mut self, path: PathBuf) {
        if path.is_dir() {
            let path = path.to_str().unwrap().to_string();

            if let Ok(h) = load_game_data_holder(&path, ChroniclesProtocol::GrandCrusade110) {
                self.holders.game_data_holder = h;
                self.edit_params.current_opened_entity = CurrentOpenedEntity::None;

                self.update_last_id();

                self.config.system_folder_path = Some(path);
                self.config.dump();
            }
        }
    }

    pub fn update_quests_java_path(&mut self, path: PathBuf) {
        if path.is_dir() {
            let path = path.to_str().unwrap().to_string();
            self.holders.server_data_holder = ServerDataHolder::load(&path);

            self.config.quest_java_classes_path = Some(path);
            self.config.dump();
        }
    }

    pub fn save_current_entity(&mut self) {
        match self.edit_params.current_opened_entity {
            CurrentOpenedEntity::Quest(index) => {
                let new_quest = self
                    .edit_params
                    .quest_edit_params
                    .opened
                    .get(index)
                    .unwrap();

                if new_quest.inner.id.0 == 0 {
                    self.show_dialog(Dialog::ShowWarning("Quest ID can't be 0!".to_string()));

                    return;
                }

                if let Some(old_quest) = self
                    .holders
                    .game_data_holder
                    .quest_holder
                    .get(&new_quest.inner.id)
                {
                    if new_quest.original_id == new_quest.inner.id {
                        self.save_quest_force(new_quest.inner.clone());
                    } else {
                        self.show_dialog(Dialog::ConfirmQuestSave {
                            message: format!(
                                "Quest with ID {} already exists.\nOverwrite?",
                                old_quest.id.0
                            ),
                            quest_id: old_quest.id,
                        });
                    }
                }
            }
            CurrentOpenedEntity::Skill(_) => {}
            CurrentOpenedEntity::None => {}
        }
    }

    pub fn save_quest_from_dlg(&mut self, quest_id: QuestId) {
        if let CurrentOpenedEntity::Quest(index) = self.edit_params.current_opened_entity {
            let new_quest = self
                .edit_params
                .quest_edit_params
                .opened
                .get(index)
                .unwrap();

            if new_quest.inner.id != quest_id {
                return;
            }

            self.save_quest_force(new_quest.inner.clone());
        }
    }

    fn save_quest_force(&mut self, mut quest: Quest) {
        if let Some(java_class) = quest.java_class {
            self.holders.server_data_holder.save_java_class(
                quest.id,
                &quest.title,
                java_class.inner,
                &self.config.quest_java_classes_path,
            )
        }

        quest.java_class = None;

        self.holders
            .game_data_holder
            .quest_holder
            .insert(quest.id, quest);
        self.filter_quests();
    }

    pub fn answer(&mut self, answer: DialogAnswer) {
        match self.dialog {
            Dialog::ConfirmQuestSave { quest_id, .. } => {
                if answer == DialogAnswer::Confirm {
                    self.save_quest_from_dlg(quest_id);
                }
            }

            Dialog::ShowWarning(_) => {}

            Dialog::None => {}
        }

        self.dialog = Dialog::None;
        self.dialog_showing = false;
    }

    fn show_dialog(&mut self, dialog: Dialog) {
        self.dialog = dialog;
        self.dialog_showing = true;
    }
}
