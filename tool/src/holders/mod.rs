use crate::backend::WindowParams;
use crate::data::{HuntingZoneId, ItemId, NpcId, QuestId};
use crate::util::l2_reader::load_dat_file;
use crate::util::{FromReader, StrUtils};
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{BufReader, Cursor};
use std::path::{Path};
use walkdir::{DirEntry, WalkDir};
use crate::entity::hunting_zone::HuntingZone;
use crate::entity::item::Item;
use crate::entity::npc::Npc;
use crate::entity::quest::{GoalType, Quest};

mod grand_crusade_110;

pub trait Loader {
    fn get_quests(&self) -> HashMap<QuestId, Quest>;
    fn get_npcs(&self) -> HashMap<NpcId, Npc>;
    fn get_npc_strings(&self) -> HashMap<u32, String>;
    fn get_items(&self) -> HashMap<ItemId, Item>;
    fn get_hunting_zones(&self) -> HashMap<HuntingZoneId, HuntingZone>;
}

fn get_loader_for_protocol(dat_paths: HashMap<String, DirEntry>, protocol: ChroniclesProtocol) -> Result<impl Loader+Sized, ()> {
    Ok(match protocol {
        ChroniclesProtocol::GrandCrusade110 => grand_crusade_110::load_holder(dat_paths)?,
    })
}

pub fn load_holder(path: &str, protocol: ChroniclesProtocol) -> Result<GameDataHolder, ()> {
    let mut dat_paths = HashMap::new();

    for entry in WalkDir::new(path) {
        if let Ok(path) = entry {
            if let Ok(meta) = path.metadata() {
                if meta.is_file() && path.file_name().to_str().unwrap().ends_with(".dat") {
                    dat_paths.insert(path.file_name().to_str().unwrap().to_lowercase(), path);
                }
            }
        }
    }

    let loader = get_loader_for_protocol(dat_paths, protocol).unwrap();

    Ok(GameDataHolder {
        protocol_version: ChroniclesProtocol::GrandCrusade110,

        npc_holder: loader.get_npcs(),
        npc_strings: loader.get_npc_strings(),
        item_holder: loader.get_items(),
        quest_holder: loader.get_quests(),
        hunting_zone_holder: loader.get_hunting_zones(),

        java_classes_holder: Default::default(),
    })

}

fn parse_dat<T: FromReader+Debug>(file_path: &Path) -> Result<Vec<T>, ()> {
    println!("Loading {file_path:?}...");
    let Ok(bytes) = load_dat_file(file_path) else {
        return Err(());
    };

    let mut reader = BufReader::new(Cursor::new(bytes));
    let count = u32::from_reader(&mut reader);

    println!("\tElements count: {count}");

    let mut res = Vec::with_capacity(count as usize);

    for _ in 0..count {
        let t = T::from_reader(&mut reader);
        res.push(t);
    }

    Ok(res)
}

#[derive(Default)]
pub enum ChroniclesProtocol {
    #[default]
    GrandCrusade110,
}

pub struct QuestInfo {
    pub(crate) id: QuestId,
    pub(crate) name: String,
}

impl Into<QuestInfo> for &Quest {
    fn into(self) -> QuestInfo {
        QuestInfo {
            id: self.id,
            name: self.title.clone()
        }
    }
}

#[derive(Default)]
pub struct GameDataHolder {
    pub protocol_version: ChroniclesProtocol,

    pub npc_holder: HashMap<NpcId, Npc>,
    pub npc_strings: HashMap<u32, String>,
    pub item_holder: HashMap<ItemId, Item>,
    pub quest_holder: HashMap<QuestId, Quest>,
    pub hunting_zone_holder: HashMap<HuntingZoneId, HuntingZone>,

    pub java_classes_holder: HashMap<QuestId, String>,
}

impl GameDataHolder {
    pub fn get_npc_name(&self, id: &NpcId) -> String {
        if let Some(npc) = self.npc_holder.get(id) {
            npc.name.clone()
        } else {
            format!("{id:?} Not Exist!")
        }
    }

    pub fn get_item_name(&self, id: &ItemId) -> String {
        if let Some(item) = self.item_holder.get(id) {
            item.name.clone()
        } else {
            format!("{id:?} Not Exist!")
        }
    }

    pub fn set_java_class(&mut self, quest: &mut Quest) {
        if let Some(v) = self.java_classes_holder.get(&quest.id) {
            quest.java_class = Some(WindowParams {
                inner: v.clone(),
                opened: false,
                action: (),
            });
        } else {
            quest.java_class = Some(WindowParams {
                inner: self.generate_java_template(quest),
                opened: false,
                action: (),
            });
        }
    }

    fn generate_java_template(&self, quest: &Quest) -> String {
        let quest_name = format!(
            "_{}_{}",
            quest.id.0,
            quest.title.deunicode().replace(' ', "")
        );
        let is_party = false; //TODO: !

        let mut start_npc_declaration = "".to_string();
        let mut start_npc_registration = "".to_string();
        let mut start_validation = "".to_string();

        for (i, npc) in quest.start_npc_ids.iter().enumerate() {
            let name = self.get_npc_name(npc).to_ascii_snake_case().to_uppercase();

            start_npc_declaration.push_str(&format!(
                "    private static final int {name} = {};\n",
                npc.0,
            ));

            start_npc_registration.push_str(&format!("        addStartNpc({name});\n",));

            start_validation.push_str(&format!(
                "npc_id == {name}{}",
                if i == quest.start_npc_ids.len() - 1 {
                    ";"
                } else {
                    " | "
                }
            ));
        }

        let mut kill_npc_declaration = "".to_string();
        let mut kill_npc_registration = "    addKillId(".to_string();

        let mut quest_items_declaration = "".to_string();
        let mut quest_items_registration = "    addQuestItem(".to_string();

        let mut proceeded_npc = vec![];
        let mut proceeded_items = vec![];

        for step in &quest.steps {
            for goal in step.inner.goals.iter() {
                match goal.goal_type {
                    GoalType::KillNpc => {
                        if proceeded_npc.contains(&goal.target_id) {
                            continue;
                        };

                        proceeded_npc.push(goal.target_id);

                        let name = self
                            .get_npc_name(&NpcId(goal.target_id))
                            .to_ascii_snake_case()
                            .to_uppercase();

                        kill_npc_declaration.push_str(&format!(
                            "    private static final int {name} = {};\n",
                            goal.target_id,
                        ));

                        kill_npc_registration.push_str(&format!(
                            "{}{name}",
                            if proceeded_npc.len() == 0 { "" } else { ", " }
                        ));
                    }

                    GoalType::CollectItem => {
                        if proceeded_items.contains(&goal.target_id) {
                            continue;
                        };

                        proceeded_items.push(goal.target_id);

                        let name = self
                            .get_item_name(&ItemId(goal.target_id))
                            .to_ascii_snake_case()
                            .to_uppercase();

                        quest_items_declaration.push_str(&format!(
                            "    private static final int {name} = {};\n",
                            goal.target_id,
                        ));

                        quest_items_registration.push_str(&format!(
                            "{}{name}",
                            if proceeded_items.len() == 0 { "" } else { ", " }
                        ));
                    }

                    _ => {}
                }
            }
        }

        if proceeded_npc.len() == 0 {
            kill_npc_registration = "".to_string();
        } else {
            kill_npc_registration.push_str(");");
        }
        if proceeded_items.len() == 0 {
            quest_items_registration = "".to_string();
        } else {
            quest_items_registration.push_str(");");
        }

        let level_check = if quest.max_lvl > 0 {
            format!("addLevelCheck({}, {});", quest.min_lvl, quest.max_lvl)
        } else {
            format!("addLevelCheck({});", quest.min_lvl)
        };

        let mut rewards_declaration = "".to_string();
        let mut give_rewards = "".to_string();

        for reward in &quest.rewards {
            let name;

            if reward.reward_id.is_adena() {
                name = "ADENA_ID".to_string();
            } else {
                name = self
                    .get_item_name(&reward.reward_id)
                    .to_ascii_snake_case()
                    .to_uppercase();

                rewards_declaration.push_str(&format!(
                    "    private static final int {name} = {};\n",
                    reward.reward_id.0,
                ));
            }

            give_rewards.push_str(&format!(
                "        quest.giveItems({name}, {});\n",
                reward.count
            ))
        }

        let class_check = if let Some(classes) = &quest.allowed_classes {
            if classes.is_empty() {
                return "".to_string();
            }

            let mut val = "addClassIdCheck(".to_string();

            for (i, class) in classes.iter().enumerate() {
                val.push_str(&format!(
                    "ClassId.{}{}",
                    class.to_string().to_ascii_snake_case(),
                    if i == classes.len() - 1 { ");\n" } else { ", " }
                ))
            }

            val
        } else {
            "".to_string()
        };

        format!(
            r#"package quests;

import org.mmocore.gameserver.model.instances.NpcInstance;
import org.mmocore.gameserver.model.quest.Quest;
import org.mmocore.gameserver.model.quest.QuestState;
import org.mmocore.gameserver.utils.ItemFunctions;

public class {quest_name} extends Quest {{
    //START NPC
{start_npc_declaration}

    //KILL NPC
{kill_npc_declaration}

    //QUEST ITEMS
{quest_items_declaration}

    //REWARDS
{rewards_declaration}


    public {quest_name}() {{
        super({is_party});
{start_npc_registration}
        {kill_npc_registration}
        {quest_items_registration}
        {level_check}
        {class_check}
    }}

    @Override
    public String onEvent(String event, QuestState st, NpcInstance npc) {{
        String htmltext = event;

        return htmltext;
    }}

    @Override
    public String onTalk(NpcInstance npc, QuestState st) {{
        String htmltext = NO_QUEST_DIALOG;
        int npcId = npc.getNpcId();
        int current_state = st.getState();
        int step = st.getCond();

        switch (current_state) {{
            case CREATED:
                if (isStartNpc(npcId)) {{
                    switch (isAvailableFor(st.getPlayer())) {{
                        //TODO:
                    }}
                }}

                break;
            case STARTED:
                //TODO:

                break;

            case COMPLETED:
                //TODO:
                break;
        }}

        return htmltext;
    }}

    @Override
    public String onKill(NpcInstance npc, QuestState quest) {{
        int npcId = npc.getNpcId();

        return null;
    }}

    private boolean isStartNpc(int npc_id) {{
        return {start_validation}
    }}

    private void giveRewards(QuestState quest) {{
{give_rewards}
    }}
}}
"#,
        )
    }
}
