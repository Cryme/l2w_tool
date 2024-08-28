use crate::backend::editor::entity::{CommonEditorOps, EntityEditParams};
use crate::backend::editor::{CurrentEntity, EditParamsCommonOps, Editors, WindowParams};
use crate::backend::entity_catalog::EntityInfo;
use crate::backend::holder::{FHashMap, HolderMapOps};
use crate::backend::{Backend, HandleAction};
use crate::common::NpcId;
use crate::entity::npc::Npc;
use serde::{Deserialize, Serialize};

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

impl Editors {
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
            self.current_entity = CurrentEntity::Npc(self.npcs.add(q.clone(), q.id, false));
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
        self.entity_catalogs.npc.filter(
            &self.holders.game_data_holder.npc_holder,
            self.entity_catalogs.filter_mode,
        );
    }

    pub fn save_npc_from_dlg(&mut self, npc_id: NpcId) {
        if let CurrentEntity::Npc(index) = self.editors.current_entity {
            let new_entity = self.editors.npcs.opened.get_mut(index).unwrap();

            if new_entity.inner.inner.id != npc_id {
                return;
            }

            new_entity.inner.initial_id = new_entity.inner.inner.id;

            let entity = new_entity.inner.inner.clone();

            new_entity.on_save();

            self.save_npc_force(entity);
        }
    }

    pub fn save_npc_force(&mut self, mut v: Npc) {
        if let Some(vv) = self.holders.game_data_holder.npc_holder.get(&v.id) {
            if *vv == v {
                return;
            }
        }

        v._changed = true;

        self.holders.game_data_holder.npc_holder.insert(v.id, v);

        self.filter_npcs();
        self.check_for_unwrote_changed();
    }
}

impl From<&Npc> for EntityInfo<Npc, NpcId> {
    fn from(value: &Npc) -> Self {
        EntityInfo::new(&format!("ID: {}\n{}", value.id.0, value.name), value)
    }
}
