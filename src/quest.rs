pub struct QuestBase {
    id: u32,
    step: u32,
    title: String,
    steps: Vec<QuestStep>,
}

pub struct Location {
    x: f32,
    y: f32,
    z: f32,
}

pub enum QuestType {
    Kill,
    Collect,
    GoTo,
    TalkTo,
}

pub struct QuestStep {
    title: String,
    desc: String,
    goals: Vec<QuestGoal>,
    location: Location,
    additional_locations: Vec<Location>,
    unk_q_level: Vec<u32>,
    min_lvl: u32,
    max_lvl: u32,
    quest_type: QuestType,
    target_display_name: String,
    get_item_in_quest: bool,
    unk_1: u32,
    unk_2: u32,
    start_npc_ids: Vec<u32>,
    start_npc_loc: Location,
    requirements: String,
    label: String,
}

pub enum GoalType {
    GetItem,
}

pub struct QuestGoal {
    target_id: u32,
    goal_type: GoalType,
    count: u32,
}
