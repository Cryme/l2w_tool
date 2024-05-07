use crate::backend::holder::FHashMap;
use crate::backend::{
    Backend, CommonEditorOps, CurrentEntity, EditParams, EntityEditParams, HandleAction,
    WindowParams,
};
use crate::data::NpcId;
use crate::entity::npc::Npc;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub type NpcEditor = EntityEditParams<Npc, NpcId, NpcAction, ()>;

impl HandleAction for WindowParams<Npc, NpcId, NpcAction, ()> {
    fn handle_action(&mut self) {
        let npc = self;
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
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum NpcMeshAction {
    #[default]
    None,
    RemoveMeshTexture(usize),
    RemoveMeshAdditionalTexture(usize),
    RemoveMeshDecoration(usize),
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum NpcSoundAction {
    #[default]
    None,
    RemoveSoundDamage(usize),
    RemoveSoundAttack(usize),
    RemoveSoundDefence(usize),
    RemoveSoundDialog(usize),
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum NpcSkillAnimationAction {
    #[default]
    None,
    RemoveSkillAnimation(usize),
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum NpcAction {
    #[default]
    None,
    RemoveProperty(usize),
    RemoveQuest(usize),
}

impl EditParams {
    pub fn get_opened_npcs_info(&self) -> Vec<(String, NpcId, bool)> {
        self.npcs.get_opened_info()
    }

    pub fn open_npc(&mut self, id: NpcId, holder: &mut FHashMap<NpcId, Npc>) {
        for (i, q) in self.npcs.opened.iter().enumerate() {
            if q.inner.initial_id == id {
                self.current_entity = CurrentEntity::Npc(i);

                return;
            }
        }

        if let Some(q) = holder.get(&id) {
            self.current_entity = CurrentEntity::Npc(self.npcs.add(q.clone(), q.id));
        }
    }

    pub fn set_current_npc(&mut self, index: usize) {
        if index < self.npcs.opened.len() {
            self.current_entity = CurrentEntity::Npc(index);
        }
    }

    pub fn create_new_npc(&mut self) {
        self.current_entity = CurrentEntity::Npc(self.npcs.add_new());
    }
}

impl Backend {
    pub fn filter_npcs(&mut self) {
        let s = self.filter_params.npc_filter_string.to_lowercase();

        let fun: Box<dyn Fn(&&Npc) -> bool> = if s.is_empty() {
            Box::new(|_: &&Npc| true)
        } else if let Ok(id) = u32::from_str(&s) {
            Box::new(move |v: &&Npc| v.id == NpcId(id))
        } else {
            Box::new(move |v: &&Npc| v.name.to_lowercase().contains(&s))
        };

        self.filter_params.npc_catalog = self
            .holders
            .game_data_holder
            .npc_holder
            .values()
            .filter(fun)
            .map(NpcInfo::from)
            .collect();

        self.filter_params
            .npc_catalog
            .sort_by(|a, b| a.id.cmp(&b.id))
    }

    pub fn save_npc_from_dlg(&mut self, npc_id: NpcId) {
        if let CurrentEntity::Npc(index) = self.edit_params.current_entity {
            let new_entity = self.edit_params.npcs.opened.get_mut(index).unwrap();

            if new_entity.inner.inner.id != npc_id {
                return;
            }

            new_entity.inner.initial_id = new_entity.inner.inner.id;

            let entity = new_entity.inner.inner.clone();

            self.save_npc_force(entity);
        }
    }

    pub(crate) fn save_npc_force(&mut self, npc: Npc) {
        if let Some(vv) = self.holders.game_data_holder.npc_holder.get(&npc.id) {
            if *vv == npc{
                return;
            }
        }
        self.set_changed();

        self.holders.game_data_holder.npc_holder.insert(npc.id, npc);

        self.filter_npcs();
    }
}

pub struct NpcInfo {
    pub(crate) id: NpcId,
    pub(crate) name: String,
}

impl From<&Npc> for NpcInfo {
    fn from(value: &Npc) -> Self {
        NpcInfo {
            id: value.id,
            name: value.name.clone(),
        }
    }
}
