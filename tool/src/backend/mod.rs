pub mod npc;
pub mod quest;
pub mod skill;
pub mod weapon;

use crate::backend::npc::{NpcAction, NpcMeshAction, NpcSkillAnimationAction, NpcSoundAction};
use crate::backend::quest::{QuestAction, StepAction};
use crate::backend::skill::{
    SkillAction, SkillEditWindowParams, SkillEnchantAction, SkillEnchantEditWindowParams,
    SkillUceConditionAction,
};
use crate::backend::weapon::WeaponAction;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::{Duration, SystemTime};

use crate::data::{ItemId, NpcId, QuestId, SkillId};
use crate::entity::item::weapon::Weapon;
use crate::entity::npc::Npc;
use crate::entity::quest::Quest;
use crate::entity::skill::{EnchantInfo, EnchantLevelInfo, Skill, SkillLevelInfo};
use crate::entity::CommonEntity;
use crate::holders::{
    get_loader_from_holder, load_game_data_holder, ChroniclesProtocol, GameDataHolder, Loader,
    NpcInfo, QuestInfo, SkillInfo, WeaponInfo,
};
use crate::server_side::ServerDataHolder;
use crate::VERSION;

const AUTO_SAVE_INTERVAL: Duration = Duration::from_secs(30);
const CONFIG_FILE_NAME: &str = "./config.ron";

#[derive(Serialize, Deserialize, Debug)]
pub struct WindowParams<Inner, OriginalId, Action, Params> {
    pub(crate) inner: Inner,
    pub(crate) opened: bool,
    pub(crate) original_id: OriginalId,
    pub(crate) action: RwLock<Action>,
    pub(crate) params: Params,
}

impl<Inner: Clone, OriginalId: Clone, Action: Default, Params: Clone> Clone
    for WindowParams<Inner, OriginalId, Action, Params>
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            opened: false,
            original_id: self.original_id.clone(),
            action: RwLock::new(Action::default()),
            params: self.params.clone(),
        }
    }
}

impl<T: Default, Action: Default> Default for WindowParams<T, (), Action, ()> {
    fn default() -> Self {
        Self {
            inner: T::default(),
            opened: false,
            original_id: (),
            action: RwLock::new(Action::default()),
            params: (),
        }
    }
}

impl<T, Action: Default> WindowParams<T, (), Action, ()> {
    pub fn new_np(inner: T) -> Self {
        Self {
            inner,
            opened: false,
            original_id: (),
            action: RwLock::new(Action::default()),
            params: (),
        }
    }
}

#[derive(Default)]
pub struct FilterParams {
    pub npc_filter_string: String,
    pub npc_catalog: Vec<NpcInfo>,

    pub quest_filter_string: String,
    pub quest_catalog: Vec<QuestInfo>,

    pub skill_filter_string: String,
    pub skill_catalog: Vec<SkillInfo>,

    pub weapon_filter_string: String,
    pub weapon_catalog: Vec<WeaponInfo>,
}

#[derive(Serialize, Deserialize, Default, Eq, PartialEq)]
pub enum CurrentOpenedEntity {
    #[default]
    None,
    Quest(usize),
    Skill(usize),
    Npc(usize),
    Weapon(usize),
}

impl CurrentOpenedEntity {
    pub fn is_some(&self) -> bool {
        *self != CurrentOpenedEntity::None
    }
}

#[derive(Serialize, Deserialize)]
pub struct EntityEditParams<Entity, EntityId, EditAction, EditParams> {
    next_id: u32,
    pub opened: Vec<WindowParams<Entity, EntityId, EditAction, EditParams>>,
}

impl<Entity, EntityId, EditAction, EditParams> Default
    for EntityEditParams<Entity, EntityId, EditAction, EditParams>
{
    fn default() -> Self {
        Self {
            next_id: 0,
            opened: vec![],
        }
    }
}

impl<
        Entity: CommonEntity<EntityId, EditParams>,
        EntityId: From<u32> + Copy + Clone,
        EditAction: Default,
        EditParams,
    > EntityEditParams<Entity, EntityId, EditAction, EditParams>
{
    fn get_opened_info(&self) -> Vec<(String, EntityId)> {
        self.opened
            .iter()
            .map(|v| (v.inner.name(), v.inner.id()))
            .collect()
    }

    fn add(&mut self, e: Entity, original_id: EntityId) -> usize {
        self.opened.push(WindowParams {
            params: e.edit_params(),
            inner: e,
            original_id,
            opened: false,
            action: RwLock::new(Default::default()),
        });

        self.opened.len() - 1
    }

    pub(crate) fn add_new(&mut self) -> usize {
        let id = EntityId::from(self.next_id);
        self.add(Entity::new(id), id);

        self.next_id += 1;

        self.opened.len() - 1
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct EditParams {
    pub npcs: EntityEditParams<Npc, NpcId, NpcAction, ()>,
    pub quests: EntityEditParams<Quest, QuestId, QuestAction, ()>,
    pub skills: EntityEditParams<Skill, SkillId, SkillAction, SkillEditWindowParams>,
    pub weapons: EntityEditParams<Weapon, ItemId, WeaponAction, ()>,
    pub current_opened_entity: CurrentOpenedEntity,
}

impl EditParams {
    fn find_opened_entity(&mut self) {
        if !self.quests.opened.is_empty() {
            self.current_opened_entity = CurrentOpenedEntity::Quest(self.quests.opened.len() - 1);
        } else if !self.skills.opened.is_empty() {
            self.current_opened_entity = CurrentOpenedEntity::Skill(self.skills.opened.len() - 1);
        } else {
            self.current_opened_entity = CurrentOpenedEntity::None;
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub system_folder_path: Option<String>,
    pub server_quests_java_classes_path: Option<String>,
    pub server_spawn_root_folder_path: Option<String>,
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
                action: RwLock::new(()),
                params: (),
            });
        } else {
            quest.java_class = Some(WindowParams {
                inner: self
                    .server_data_holder
                    .generate_java_template(quest, &self.game_data_holder),
                original_id: (),
                opened: false,
                action: RwLock::new(()),
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
    ConfirmNpcSave { message: String, npc_id: NpcId },
    ConfirmWeaponSave { message: String, weapon_id: ItemId },
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

        let server_data_holder = if let Some(path) = &config.server_quests_java_classes_path {
            ServerDataHolder::load(path)
        } else {
            ServerDataHolder::default()
        };

        let edit_params = if let Ok(f) = File::open(format!("./v{VERSION}.asave")) {
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
            filter_params: FilterParams::default(),
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
        self.filter_params.npc_filter_string = "".to_string();
        self.filter_npcs();
        self.filter_params.weapon_filter_string = "".to_string();
        self.filter_weapons();

        self.edit_params.quests.next_id =
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
        self.edit_params.npcs.next_id = if let Some(last) = self.filter_params.npc_catalog.last() {
            last.id.0 + 1
        } else {
            0
        };
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

        if let Ok(mut out) = File::create(format!("./v{VERSION}.asave")) {
            out.write_all(&bincode::serialize(&self.edit_params).unwrap())
                .unwrap();
        }

        self.tasks.last_auto_save = SystemTime::now();
    }

    fn proceed_actions(&mut self) {
        match self.edit_params.current_opened_entity {
            CurrentOpenedEntity::Npc(index) => {
                let npc = &mut self.edit_params.npcs.opened[index];
                {
                    let mut action = npc.action.write().unwrap();

                    match *action {
                        NpcAction::RemoveProperty(i) => {
                            npc.inner.properties.remove(i);
                        }
                        NpcAction::RemoveQuest(i) => {
                            npc.inner.quest_infos.remove(i);
                        }

                        NpcAction::None => {}
                    }

                    *action = NpcAction::None;
                }

                {
                    let mut action = npc.inner.mesh_params.action.write().unwrap();

                    match *action {
                        NpcMeshAction::RemoveMeshTexture(i) => {
                            npc.inner.mesh_params.inner.textures.remove(i);
                        }
                        NpcMeshAction::RemoveMeshAdditionalTexture(i) => {
                            npc.inner.mesh_params.inner.additional_textures.remove(i);
                        }
                        NpcMeshAction::RemoveMeshDecoration(i) => {
                            npc.inner.mesh_params.inner.decorations.remove(i);
                        }

                        NpcMeshAction::None => {}
                    }

                    *action = NpcMeshAction::None;
                }

                {
                    let mut action = npc.inner.sound_params.action.write().unwrap();

                    match *action {
                        NpcSoundAction::RemoveSoundDamage(i) => {
                            npc.inner.sound_params.inner.damage_sound.remove(i);
                        }
                        NpcSoundAction::RemoveSoundAttack(i) => {
                            npc.inner.sound_params.inner.attack_sound.remove(i);
                        }
                        NpcSoundAction::RemoveSoundDefence(i) => {
                            npc.inner.sound_params.inner.defence_sound.remove(i);
                        }
                        NpcSoundAction::RemoveSoundDialog(i) => {
                            npc.inner.sound_params.inner.dialog_sound.remove(i);
                        }

                        NpcSoundAction::None => {}
                    }

                    *action = NpcSoundAction::None;
                }

                {
                    let mut action = npc.inner.skill_animations.action.write().unwrap();

                    match *action {
                        NpcSkillAnimationAction::RemoveSkillAnimation(i) => {
                            npc.inner.skill_animations.inner.remove(i);
                        }

                        NpcSkillAnimationAction::None => {}
                    }

                    *action = NpcSkillAnimationAction::None;
                }
            }

            CurrentOpenedEntity::Quest(index) => {
                let quest = &mut self.edit_params.quests.opened[index];
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

            CurrentOpenedEntity::Skill(index) => {
                let skill = &mut self.edit_params.skills.opened[index];

                let mut action = skill.action.write().unwrap();

                match *action {
                    SkillAction::DeleteLevel => {
                        skill
                            .inner
                            .skill_levels
                            .remove(skill.params.current_level_index);

                        for level in
                            &mut skill.inner.skill_levels[skill.params.current_level_index..]
                        {
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
                        let curr_level =
                            &mut skill.inner.skill_levels[skill.params.current_level_index];

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
                        let curr_level =
                            &mut skill.inner.skill_levels[skill.params.current_level_index];
                        curr_level.available_enchants.remove(index);
                    }
                    SkillAction::AddEnchantLevel(index) => {
                        let curr_enchant = &mut skill.inner.skill_levels
                            [skill.params.current_level_index]
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
                        let curr_enchant = &mut skill.inner.skill_levels
                            [skill.params.current_level_index]
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

            CurrentOpenedEntity::Weapon(index) => {}

            CurrentOpenedEntity::None => {}
        }
    }

    pub fn on_update(&mut self) {
        self.proceed_actions();
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

            self.config.server_quests_java_classes_path = Some(path);
            self.config.dump();
        }
    }

    pub fn update_npc_spawn_path(&mut self, path: PathBuf) {
        if path.is_dir() {
            let path = path.to_str().unwrap().to_string();
            self.config.server_spawn_root_folder_path = Some(path);
            self.config.dump();
        }
    }

    pub fn save_current_entity(&mut self) {
        match self.edit_params.current_opened_entity {
            CurrentOpenedEntity::Npc(index) => {
                let new_npc = self.edit_params.npcs.opened.get(index).unwrap();

                if new_npc.inner.id.0 == 0 {
                    self.show_dialog(Dialog::ShowWarning("Npc ID can't be 0!".to_string()));

                    return;
                }

                if let Some(old_npc) = self
                    .holders
                    .game_data_holder
                    .npc_holder
                    .get(&new_npc.inner.id)
                {
                    if new_npc.original_id == new_npc.inner.id {
                        self.save_npc_force(new_npc.inner.clone());
                    } else {
                        self.show_dialog(Dialog::ConfirmNpcSave {
                            message: format!(
                                "Npc with ID {} already exists.\nOverwrite?",
                                old_npc.id.0
                            ),
                            npc_id: old_npc.id,
                        });
                    }
                } else {
                    self.save_npc_force(new_npc.inner.clone());
                }
            }

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
                } else {
                    self.save_quest_force(new_quest.inner.clone());
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
                } else {
                    self.save_skill_force(new_skill.inner.clone());
                }
            }

            CurrentOpenedEntity::Weapon(index) => {
                let new_entity = self.edit_params.weapons.opened.get(index).unwrap();

                if new_entity.inner.id().0 == 0 {
                    self.show_dialog(Dialog::ShowWarning("Item ID can't be 0!".to_string()));

                    return;
                }

                if let Some(old_entity) = self
                    .holders
                    .game_data_holder
                    .weapon_holder
                    .get(&new_entity.inner.id())
                {
                    if new_entity.original_id == new_entity.inner.id() {
                        self.save_weapon_force(new_entity.inner.clone());
                    } else {
                        self.show_dialog(Dialog::ConfirmWeaponSave {
                            message: format!(
                                "Weapon with ID {} already exists.\nOverwrite?",
                                old_entity.id().0
                            ),
                            weapon_id: old_entity.id(),
                        });
                    }
                } else {
                    self.save_weapon_force(new_entity.inner.clone());
                }
            }

            CurrentOpenedEntity::None => {}
        }
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

            Dialog::ConfirmNpcSave { npc_id, .. } => {
                if answer == DialogAnswer::Confirm {
                    self.save_npc_from_dlg(npc_id);
                }
            }

            Dialog::ConfirmWeaponSave { weapon_id, .. } => {
                if answer == DialogAnswer::Confirm {
                    self.save_weapon_from_dlg(weapon_id);
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
