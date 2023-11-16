use crate::data::{QuestId, SkillId};
use crate::entity::quest::Quest;
use crate::entity::skill::{EnchantInfo, EnchantLevelInfo, Skill, SkillLevelInfo};
use crate::holders::{
    get_loader_from_holder, load_game_data_holder, ChroniclesProtocol, GameDataHolder, Loader,
    QuestInfo, SkillInfo,
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

#[derive(Serialize, Deserialize, Debug)]
pub enum QuestAction {
    None,
    RemoveStep(usize),
    RemoveStartNpcId(usize),
    RemoveReward(usize),
    RemoveQuestItem(usize),
}

#[derive(Serialize, Deserialize, Clone)]
pub enum SkillAction {
    None,
    DeleteLevel,
    AddLevel,
    AddEnchant,
    DeleteEnchant(usize),
    AddEnchantLevel(usize),
    DeleteEnchantLevel(usize),
}

#[derive(Serialize, Deserialize, Clone)]
pub enum SkillEnchantAction {
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WindowParams<Inner, OriginalId, Action, Params> {
    pub(crate) inner: Inner,
    pub(crate) opened: bool,
    pub(crate) original_id: OriginalId,
    pub(crate) action: Action,
    pub(crate) params: Params,
}

pub struct FilterParams {
    pub quest_filter_string: String,
    pub quest_catalog: Vec<QuestInfo>,
    pub skill_filter_string: String,
    pub skill_catalog: Vec<SkillInfo>,
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
pub struct SkillEditWindowParams {
    pub current_level_index: usize,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct SkillEnchantEditWindowParams {
    pub current_level_index: usize,
}

#[derive(Serialize, Deserialize, Default)]
pub struct SkillEditParams {
    next_id: u32,
    pub opened: Vec<WindowParams<Skill, SkillId, SkillAction, SkillEditWindowParams>>,
}

impl SkillEditParams {
    fn get_opened_info(&self) -> Vec<(String, SkillId)> {
        self.opened
            .iter()
            .map(|v| (v.inner.name.clone(), v.inner.id))
            .collect()
    }

    fn add(&mut self, e: Skill, original_id: SkillId) -> usize {
        let current_level_index = e.skill_levels.len() - 1;

        self.opened.push(WindowParams {
            inner: e,
            original_id,
            opened: false,
            action: SkillAction::None,
            params: SkillEditWindowParams {
                current_level_index,
            },
        });

        self.opened.len() - 1
    }

    fn add_new(&mut self) -> usize {
        // self.add(Skill::new(self.next_id), SkillId(self.next_id));

        self.next_id += 1;

        self.opened.len() - 1
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct QuestEditParams {
    next_quest_id: u32,
    pub opened: Vec<WindowParams<Quest, QuestId, QuestAction, ()>>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct EditParams {
    pub quests: QuestEditParams,
    pub skills: SkillEditParams,
    pub current_opened_entity: CurrentOpenedEntity,
}

impl EditParams {
    pub fn get_opened_quests_info(&self) -> Vec<(String, QuestId)> {
        self.quests.get_opened_quests_info()
    }
    pub fn get_opened_skills_info(&self) -> Vec<(String, SkillId)> {
        self.skills.get_opened_info()
    }

    pub fn open_quest(&mut self, id: QuestId, holder: &mut HashMap<QuestId, Quest>) {
        for (i, q) in self.quests.opened.iter().enumerate() {
            if q.original_id == id {
                self.current_opened_entity = CurrentOpenedEntity::Quest(i);

                return;
            }
        }

        if let Some(q) = holder.get(&id) {
            self.current_opened_entity =
                CurrentOpenedEntity::Quest(self.quests.add_quest(q.clone(), q.id));
        }
    }

    pub fn open_skill(&mut self, id: SkillId, holder: &mut HashMap<SkillId, Skill>) {
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

    pub fn set_current_quest(&mut self, index: usize) {
        if index < self.quests.opened.len() {
            self.current_opened_entity = CurrentOpenedEntity::Quest(index);
        }
    }

    pub fn set_current_skill(&mut self, index: usize) {
        if index < self.skills.opened.len() {
            self.current_opened_entity = CurrentOpenedEntity::Skill(index);
        }
    }

    pub fn close_quest(&mut self, index: usize) {
        self.quests.opened.remove(index);

        if let CurrentOpenedEntity::Quest(curr_index) = self.current_opened_entity {
            if self.quests.opened.is_empty() {
                self.find_opened_entity();
            } else if curr_index >= index {
                self.current_opened_entity = CurrentOpenedEntity::Quest(curr_index.max(1) - 1)
            }
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

    fn find_opened_entity(&mut self) {
        if !self.quests.opened.is_empty() {
            self.current_opened_entity = CurrentOpenedEntity::Quest(self.quests.opened.len() - 1);
        } else if !self.skills.opened.is_empty() {
            self.current_opened_entity = CurrentOpenedEntity::Skill(self.skills.opened.len() - 1);
        } else {
            self.current_opened_entity = CurrentOpenedEntity::None;
        }
    }

    pub fn create_new_quest(&mut self) {
        self.current_opened_entity = CurrentOpenedEntity::Quest(self.quests.add_new_quest());
    }

    pub fn create_new_skill(&mut self) {
        self.current_opened_entity = CurrentOpenedEntity::Skill(self.skills.add_new());
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
            params: (),
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
                params: (),
            });
        } else {
            quest.java_class = Some(WindowParams {
                inner: self
                    .server_data_holder
                    .generate_java_template(quest, &self.game_data_holder),
                original_id: (),
                opened: false,
                action: (),
                params: (),
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
    ConfirmSkillSave { message: String, skill_id: SkillId },
    ShowWarning(String),
}

impl Backend {
    pub(crate) fn save_to_dat(&self) {
        let mut loader = get_loader_from_holder(&self.holders.game_data_holder);

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
                skill_filter_string: "".to_string(),
                skill_catalog: vec![],
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
        self.filter_params.skill_filter_string = "".to_string();
        self.filter_skills();

        self.edit_params.quests.next_quest_id =
            if let Some(last) = self.filter_params.quest_catalog.last() {
                last.id.0 + 1
            } else {
                0
            };
        self.edit_params.skills.next_id =
            if let Some(last) = self.filter_params.skill_catalog.last() {
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

    pub fn filter_skills(&mut self) {
        let mut s = self.filter_params.skill_filter_string.clone();

        let fun: Box<dyn Fn(&&Skill) -> bool> = if s.is_empty() {
            Box::new(|_: &&Skill| true)
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
                let quest = &mut self.edit_params.quests.opened[index];
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
            CurrentOpenedEntity::Skill(index) => {
                let skill = &mut self.edit_params.skills.opened[index];

                match skill.action {
                    SkillAction::DeleteLevel => {
                        skill.inner.skill_levels.remove(skill.params.current_level_index);
                        skill.params.current_level_index = skill.params.current_level_index.min(skill.inner.skill_levels.len() - 1).max(0)
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
                                    action: SkillEnchantAction::None,
                                    params: SkillEnchantEditWindowParams{ current_level_index: v.inner.enchant_levels.len() - 1 },
                                }
                            } else {
                                WindowParams {
                                    inner: EnchantInfo::default(),
                                    opened: false,
                                    original_id: (),
                                    action: SkillEnchantAction::None,
                                    params: SkillEnchantEditWindowParams{ current_level_index: 0 },
                                }
                            }
                        )
                    }
                    SkillAction::DeleteEnchant(index) => {
                        let curr_level = &mut skill.inner.skill_levels[skill.params.current_level_index];
                        curr_level.available_enchants.remove(index);
                    }
                    SkillAction::AddEnchantLevel(index) => {
                        let curr_enchant = &mut skill.inner.skill_levels[skill.params.current_level_index].available_enchants[index];
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
                            curr_enchant.inner.enchant_levels.insert(proto_index, new_level);
                            curr_enchant.params.current_level_index = proto_index;
                        } else {
                            curr_enchant.params.current_level_index = curr_enchant.inner.enchant_levels.len();
                            curr_enchant.inner.enchant_levels.push(new_level);
                        }
                    }
                    SkillAction::DeleteEnchantLevel(index) => {
                        let curr_enchant = &mut skill.inner.skill_levels[skill.params.current_level_index].available_enchants[index];
                        curr_enchant.inner.enchant_levels.remove(curr_enchant.params.current_level_index);
                        curr_enchant.params.current_level_index = curr_enchant.params.current_level_index.min(curr_enchant.inner.enchant_levels.len() - 1).max(0)
                    }

                    SkillAction::None => {}
                }

                skill.action = SkillAction::None;
            }
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
                let new_quest = self.edit_params.quests.opened.get(index).unwrap();

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
            CurrentOpenedEntity::Skill(index) => {
                let new_skill = self.edit_params.skills.opened.get(index).unwrap();

                if new_skill.inner.id.0 == 0 {
                    self.show_dialog(Dialog::ShowWarning("Skill ID can't be 0!".to_string()));

                    return;
                }

                if let Some(old_skill) = self
                    .holders
                    .game_data_holder
                    .skill_holder
                    .get(&new_skill.inner.id)
                {
                    if new_skill.original_id == new_skill.inner.id {
                        self.save_skill_force(new_skill.inner.clone());
                    } else {
                        self.show_dialog(Dialog::ConfirmSkillSave {
                            message: format!(
                                "Skill with ID {} already exists.\nOverwrite?",
                                old_skill.id.0
                            ),
                            skill_id: old_skill.id,
                        });
                    }
                }
            }
            CurrentOpenedEntity::None => {}
        }
    }

    pub fn save_quest_from_dlg(&mut self, quest_id: QuestId) {
        if let CurrentOpenedEntity::Quest(index) = self.edit_params.current_opened_entity {
            let new_quest = self.edit_params.quests.opened.get(index).unwrap();

            if new_quest.inner.id != quest_id {
                return;
            }

            self.save_quest_force(new_quest.inner.clone());
        }
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

    fn save_skill_force(&mut self, skill: Skill) {
        self.holders
            .game_data_holder
            .skill_holder
            .insert(skill.id, skill);
        self.filter_skills();
    }

    pub fn answer(&mut self, answer: DialogAnswer) {
        match self.dialog {
            Dialog::ConfirmQuestSave { quest_id, .. } => {
                if answer == DialogAnswer::Confirm {
                    self.save_quest_from_dlg(quest_id);
                }
            }

            Dialog::ConfirmSkillSave { skill_id, .. } => {
                if answer == DialogAnswer::Confirm {
                    self.save_skill_from_dlg(skill_id);
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
