use crate::data::QuestId;
use crate::entity::quest::Quest;
use crate::holders::{load_game_data_holder, ChroniclesProtocol, GameDataHolder, QuestInfo};
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

#[derive(Serialize, Deserialize, Default)]
pub struct QuestEditParams {
    next_quest_id: u32,
    pub current_quest: Option<usize>,
    pub opened_quests: Vec<WindowParams<Quest, QuestId, QuestAction>>,
}

impl QuestEditParams {
    pub fn get_opened_quests_info(&self) -> Vec<(String, QuestId)> {
        self.opened_quests
            .iter()
            .map(|v| (v.inner.title.clone(), v.inner.id))
            .collect()
    }
    pub fn open_quest(&mut self, id: QuestId, holder: &mut HashMap<QuestId, Quest>) {
        for (i, q) in self.opened_quests.iter().enumerate() {
            if q.original_id == id {
                self.current_quest = Some(i);

                return;
            }
        }

        if let Some(q) = holder.get(&id) {
            self.add_quest(q.clone_and_replace_escaped(), q.id);
        }
    }

    pub fn set_current_quest(&mut self, index: usize) {
        if index < self.opened_quests.len() {
            self.current_quest = Some(index);
        }
    }

    pub fn close_quest(&mut self, index: usize) {
        self.opened_quests.remove(index);

        if let Some(curr_index) = self.current_quest {
            if self.opened_quests.is_empty() {
                self.current_quest = None
            } else if curr_index >= index {
                self.current_quest = Some(curr_index.max(1) - 1)
            }
        }
    }

    pub fn get_current_quest(&mut self) -> Option<&mut WindowParams<Quest, QuestId, QuestAction>> {
        if let Some(index) = self.current_quest {
            return Some(&mut self.opened_quests[index]);
        }

        None
    }

    fn add_quest(&mut self, quest: Quest, original_id: QuestId) {
        self.opened_quests.push(WindowParams {
            inner: quest,
            original_id,
            opened: false,
            action: QuestAction::None,
        });

        self.current_quest = Some(self.opened_quests.len() - 1);
    }

    pub fn create_new_quest(&mut self) {
        self.add_quest(Quest::new(self.next_quest_id), QuestId(self.next_quest_id));

        self.next_quest_id += 1;
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
    pub quest_edit_params: QuestEditParams,
    pub holders: Holders,
    pub filter_params: FilterParams,

    tasks: Tasks,
}

impl Backend {
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

        let quest_edit_params = if let Ok(f) = File::open(AUTO_SAVE_FILE_NAME) {
            if let Ok(d) = bincode::deserialize_from::<File, QuestEditParams>(f) {
                d
            } else {
                QuestEditParams::default()
            }
        } else {
            QuestEditParams::default()
        };

        let mut r = Self {
            config,
            quest_edit_params,

            holders: Holders {
                game_data_holder,
                server_data_holder,
            },
            filter_params: FilterParams {
                quest_filter_string: "".to_string(),
                quest_catalog: vec![],
            },
            tasks: Tasks::init(),
        };

        r.update_last_id();

        r
    }

    fn update_last_id(&mut self) {
        self.filter_params.quest_filter_string = "".to_string();
        self.filter_quests();

        self.quest_edit_params.next_quest_id =
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

    fn auto_save(&mut self) {
        if SystemTime::now()
            .duration_since(self.tasks.last_auto_save)
            .unwrap()
            < AUTO_SAVE_INTERVAL
        {
            return;
        }

        if let Ok(mut out) = File::create(AUTO_SAVE_FILE_NAME) {
            out.write_all(&bincode::serialize(&self.quest_edit_params).unwrap())
                .unwrap();
        }

        self.tasks.last_auto_save = SystemTime::now();
    }

    fn remove_deleted(&mut self) {
        if let Some(index) = self.quest_edit_params.current_quest {
            let quest = &mut self.quest_edit_params.opened_quests[index];
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

    pub fn on_update(&mut self) {
        self.remove_deleted();
        self.auto_save();
    }

    pub fn update_system_path(&mut self, path: PathBuf) {
        if path.is_dir() {
            let path = path.to_str().unwrap().to_string();

            if let Ok(h) = load_game_data_holder(&path, ChroniclesProtocol::GrandCrusade110) {
                self.holders.game_data_holder = h;
                self.quest_edit_params.current_quest = None;

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
}
