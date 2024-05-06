use crate::backend::dat_loader::StrUtils;
use crate::backend::holder::GameDataHolder;
use crate::backend::Config;
use crate::data::{ItemId, NpcId, QuestId};
use crate::entity::quest::{GoalType, Quest};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use walkdir::{DirEntry, WalkDir};

#[derive(Default)]
pub struct ServerDataHolder {
    pub quest_java_classes: HashMap<QuestId, DirEntry>,
}

impl ServerDataHolder {
    pub fn validate_paths(config: &mut Config) {
        if let Some(path) = &config.server_quests_java_classes_path {
            if !Path::new(path).is_dir() {
                config.server_quests_java_classes_path = None
            }
        }
    }

    pub fn load(path: &String) -> Self {
        let mut quest_java_classes = HashMap::new();

        for path in WalkDir::new(path).into_iter().flatten() {
            let Ok(meta) = path.metadata() else { continue };

            if !meta.is_file() {
                continue;
            }

            let file_name = path.file_name().to_str().unwrap();
            if !file_name.ends_with(".java") {
                continue;
            }

            let Some(id) = file_name.split('_').nth(1) else {
                continue;
            };
            let Ok(id) = u32::from_str(id) else { continue };

            quest_java_classes.insert(QuestId(id), path);
        }

        Self { quest_java_classes }
    }

    pub fn save_java_class(
        &mut self,
        id: QuestId,
        quest_title: &String,
        java_class: String,
        quest_dir_path: &Option<String>,
    ) {
        if let Some(path) = self.quest_java_classes.get(&id) {
            if let Ok(mut out) = File::create(path.path()) {
                out.write_all(java_class.as_ref()).unwrap();
            }
        } else if let Some(path) = quest_dir_path {
            let path = Path::new(path).join(format!(
                "_{}_{}.java",
                id.0,
                quest_title.to_ascii_camel_case()
            ));

            if let Ok(mut out) = File::create(path) {
                out.write_all(java_class.as_ref()).unwrap();
            }
        }
    }

    pub(crate) fn generate_java_template(
        &self,
        quest: &Quest,
        game_data_holder: &GameDataHolder,
    ) -> String {
        let quest_name = format!("_{}_{}", quest.id.0, quest.title.to_ascii_camel_case());
        let is_party = false; //TODO: !

        let mut start_npc_declaration = "".to_string();
        let mut start_npc_registration = "".to_string();
        let mut start_validation = "".to_string();

        for (i, npc) in quest.start_npc_ids.iter().enumerate() {
            let name = game_data_holder
                .get_npc_name(npc)
                .to_ascii_snake_case()
                .to_uppercase();

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
        let mut kill_npc_registration = "addKillId(".to_string();

        let mut quest_items_declaration = "".to_string();
        let mut quest_items_registration = "addQuestItem(".to_string();

        let mut proceeded_npc = vec![];
        let mut proceeded_items = vec![];

        for step in &quest.steps {
            for goal in step.inner.goals.iter() {
                match goal.goal_type {
                    GoalType::KillNpc => {
                        if proceeded_npc.contains(&goal.target_id) {
                            continue;
                        };

                        let name = game_data_holder
                            .get_npc_name(&NpcId(goal.target_id))
                            .to_ascii_snake_case()
                            .to_uppercase();

                        kill_npc_declaration.push_str(&format!(
                            "    private static final int {name} = {};\n",
                            goal.target_id,
                        ));

                        kill_npc_registration.push_str(&format!(
                            "{}{name}",
                            if proceeded_npc.is_empty() { "" } else { ", " }
                        ));

                        proceeded_npc.push(goal.target_id);
                    }

                    GoalType::CollectItem => {
                        if proceeded_items.contains(&goal.target_id) {
                            continue;
                        };

                        let name = game_data_holder
                            .get_item_name(&ItemId(goal.target_id))
                            .to_ascii_snake_case()
                            .to_uppercase();

                        quest_items_declaration.push_str(&format!(
                            "    private static final int {name} = {};\n",
                            goal.target_id,
                        ));

                        quest_items_registration.push_str(&format!(
                            "{}{name}",
                            if proceeded_items.is_empty() { "" } else { ", " }
                        ));

                        proceeded_items.push(goal.target_id);
                    }

                    _ => {}
                }
            }
        }

        if proceeded_npc.is_empty() {
            kill_npc_registration = "".to_string();
        } else {
            kill_npc_registration.push_str(");");
        }
        if proceeded_items.is_empty() {
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
                name = game_data_holder
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
