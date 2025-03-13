use crate::backend::dat_loader::grand_crusade_166::{Collision, Color, L2GeneralStringTable};
use crate::backend::editor::WindowParams;
use crate::common::{ItemId, NpcId, QuestId, SkillId};
use crate::entity::npc::{
    Npc, NpcAdditionalParts, NpcDecorationEffect, NpcEquipParams, NpcMeshParams, NpcProperty,
    NpcQuestInfo, NpcSkillAnimation, NpcSoundParams, NpcSummonParams, SummonType,
};

use l2_rw::ue2_rw::{ASCF, BYTE, DOUBLE, DWORD, FLOAT, USHORT, UVEC};
use l2_rw::{deserialize_dat, save_dat, DatVariant};

use l2_rw::ue2_rw::{ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal};

use crate::backend::dat_loader::{wrap_into_id_map, wrap_into_id_vec_map, GetId};
use crate::backend::holder::{GameDataHolder, HolderMapOps};
use crate::backend::log_holder::{Log, LogLevel};
use eframe::egui::Color32;
use num_traits::{FromPrimitive, ToPrimitive};
use r#macro::{ReadUnreal, WriteUnreal};
use std::collections::HashMap;
use std::thread;
use std::thread::JoinHandle;

impl MobSkillAnimGrpDat {
    fn from(v: (&Npc, &mut L2GeneralStringTable)) -> Vec<Self> {
        let npc = v.0;
        let table = v.1;
        let mut res = vec![];

        for v in &npc.skill_animations.inner {
            res.push(Self {
                npc_id: npc.id.0,
                skill_id: v.id.0,
                animation: table.get_index(&v.animation),
            })
        }

        res
    }
}
impl From<(&Npc, &mut L2GeneralStringTable)> for NpcNameDat {
    fn from(v: (&Npc, &mut L2GeneralStringTable)) -> Self {
        let npc = v.0;

        Self {
            id: npc.id.0,
            name: (&npc.name).into(),
            title: (&npc.title).into(),
            title_color: Color {
                r: npc.title_color.r(),
                g: npc.title_color.g(),
                b: npc.title_color.b(),
                a: npc.title_color.a(),
            },
        }
    }
}

impl AdditionalNpcGrpPartsDat {
    fn from(v: (&Npc, &mut L2GeneralStringTable)) -> Option<Self> {
        let npc = v.0;
        let table = v.1;

        let Some(parts) = &npc.additional_parts.inner else {
            return None;
        };

        Some(Self {
            npc_id: npc.id.0,
            class: table.get_index(&parts.class),
            chest: parts.chest.0,
            legs: parts.legs.0,
            gloves: parts.gloves.0,
            feet: parts.feet.0,
            back: parts.back.0,
            hair_accessory: parts.hair_accessory.0,
            hair_style: parts.hair_style,
            right_hand: parts.right_hand.0,
            left_hand: parts.left_hand.0,
        })
    }
}

impl From<(&Npc, &mut L2GeneralStringTable)> for NpcGrpDat {
    fn from(v: (&Npc, &mut L2GeneralStringTable)) -> Self {
        let npc = v.0;
        let table = v.1;

        Self {
            id: npc.id.0 as USHORT,
            unreal_class: table.get_index(&npc.unreal_script_class),
            mesh: table.get_index(&npc.mesh_params.inner.mesh),
            texture_1: npc
                .mesh_params
                .inner
                .textures
                .iter()
                .map(|v| table.get_index(v))
                .collect(),
            texture_2: UVEC::from(
                npc.mesh_params
                    .inner
                    .textures
                    .iter()
                    .map(|v| table.get_index(v))
                    .collect::<Vec<DWORD>>(),
            ),
            properties: npc
                .properties
                .iter()
                .flat_map(|e| [e.id.0 as u16, e.level])
                .collect(),
            npc_speed: npc.mesh_params.inner.speed,
            attack_sound: npc
                .sound_params
                .inner
                .attack_sound
                .iter()
                .map(|v| table.get_index(v))
                .collect(),
            defence_sound: npc
                .sound_params
                .inner
                .defence_sound
                .iter()
                .map(|v| table.get_index(v))
                .collect(),
            damage_sound: npc
                .sound_params
                .inner
                .damage_sound
                .iter()
                .map(|v| table.get_index(v))
                .collect(),
            deco_effect: npc
                .mesh_params
                .inner
                .decorations
                .iter()
                .map(|v| DecoEffect {
                    effect: table.get_index(&v.effect),
                    scale: v.scale,
                })
                .collect(),
            quests: npc
                .quest_infos
                .iter()
                .map(|v| NpcQuestData {
                    id: v.id.0 as USHORT,
                    step: v.step,
                })
                .collect(),
            attack_effect: table.get_index(&npc.mesh_params.inner.attack_effect),
            sound_vol: npc.sound_params.inner.vol,
            sound_radius: npc.sound_params.inner.rad,
            sound_random: npc.sound_params.inner.random,
            social: npc.social.into(),
            show_hp: npc.show_hp.into(),
            dialog_sounds: UVEC::from(
                npc.sound_params
                    .inner
                    .dialog_sound
                    .iter()
                    .map(|v| table.get_index(v))
                    .collect::<Vec<DWORD>>(),
            ),
            silhouette: npc.summon_params.inner.silhouette,
            summon_sort: npc.summon_params.inner.summon_type.to_u8().unwrap(),
            summon_max_count: npc.summon_params.inner.max_count,
            summon_grade: npc.summon_params.inner.grade,
            draw_scale: npc.mesh_params.inner.draw_scale,
            use_zoom_in_cam: npc.mesh_params.inner.use_zoomincam,
            npc_icon: table.get_index(&npc.icon),
            sound_priority: npc.sound_params.inner.priority,
            run_speed: npc.mesh_params.inner.run_speed,
            walk_speed: npc.mesh_params.inner.walk_speed,
            collision: Collision {
                radius_1: npc.mesh_params.inner.collision_radius_1,
                radius_2: npc.mesh_params.inner.collision_radius_2,
                height_1: npc.mesh_params.inner.collision_height_1,
                height_2: npc.mesh_params.inner.collision_height_2,
            },
            left_hand: npc.equipment_params.inner.left_hand.0,
            right_hand: npc.equipment_params.inner.left_hand.0,
            chest: npc.equipment_params.inner.left_hand.0,
            hp: npc.org_hp,
            mp: npc.org_mp,
            npc_type: npc.npc_type,
        }
    }
}

impl GameDataHolder {
    pub fn serialize_npcs_to_binary(&mut self) -> JoinHandle<Vec<Log>> {
        let mut logs = vec![];

        let mut npc_grp: Vec<NpcGrpDat> = vec![];
        let mut additional_npc_parts_grp: Vec<AdditionalNpcGrpPartsDat> = vec![];
        let mut npc_name: Vec<NpcNameDat> = vec![];
        let mut mob_skill_anim: Vec<MobSkillAnimGrpDat> = vec![];

        for npc in self.npc_holder.values().filter(|v| !v._deleted) {
            npc_grp.push((npc, &mut self.game_string_table).into());

            if let Some(v) = AdditionalNpcGrpPartsDat::from((npc, &mut self.game_string_table)) {
                additional_npc_parts_grp.push(v);
            }

            npc_name.push((npc, &mut self.game_string_table).into());

            mob_skill_anim.extend(MobSkillAnimGrpDat::from((npc, &mut self.game_string_table)));
        }

        let npc_grp_path = self
            .dat_paths
            .get(&"npcgrp.dat".to_string())
            .unwrap()
            .clone();

        let additional_npc_parts_path = self
            .dat_paths
            .get(&"additionalnpcgrpparts.dat".to_string())
            .unwrap()
            .clone();

        let npc_name_path = self
            .dat_paths
            .get(&"npcname-ru.dat".to_string())
            .unwrap()
            .clone();

        let mob_skill_anim_path = self
            .dat_paths
            .get(&"mobskillanimgrp.dat".to_string())
            .unwrap()
            .clone();

        thread::spawn(move || {
            let npc_grp_handle = thread::spawn(move || {
                if let Err(e) = save_dat(
                    npc_grp_path.path(),
                    DatVariant::<(), NpcGrpDat>::Array(npc_grp.to_vec()),
                ) {
                    Log::from_loader_e(e)
                } else {
                    Log::from_loader_i("NpcGrp saved")
                }
            });
            let additional_npc_parts_handle = thread::spawn(move || {
                if let Err(e) = save_dat(
                    additional_npc_parts_path.path(),
                    DatVariant::<(), AdditionalNpcGrpPartsDat>::Array(
                        additional_npc_parts_grp.to_vec(),
                    ),
                ) {
                    Log::from_loader_e(e)
                } else {
                    Log::from_loader_i("AdditionalNpcPartsGrp saved")
                }
            });
            let npc_name_handle = thread::spawn(move || {
                if let Err(e) = save_dat(
                    npc_name_path.path(),
                    DatVariant::<(), NpcNameDat>::Array(npc_name.to_vec()),
                ) {
                    Log::from_loader_e(e)
                } else {
                    Log::from_loader_i("NpcName saved")
                }
            });
            let mob_skill_anim_handle = thread::spawn(move || {
                if let Err(e) = save_dat(
                    mob_skill_anim_path.path(),
                    DatVariant::<(), MobSkillAnimGrpDat>::Array(mob_skill_anim.to_vec()),
                ) {
                    Log::from_loader_e(e)
                } else {
                    Log::from_loader_i("MobSkillAnimGrp saved")
                }
            });

            logs.push(npc_grp_handle.join().unwrap());
            logs.push(additional_npc_parts_handle.join().unwrap());
            logs.push(npc_name_handle.join().unwrap());
            logs.push(mob_skill_anim_handle.join().unwrap());

            logs
        })
    }

    pub fn load_npcs(&mut self) -> Result<Vec<Log>, ()> {
        let npc_grp = deserialize_dat::<NpcGrpDat>(
            self.dat_paths
                .get(&"npcgrp.dat".to_string())
                .unwrap()
                .path(),
        )?;

        let mut npc_additional_parts_grp =
            wrap_into_id_map(deserialize_dat::<AdditionalNpcGrpPartsDat>(
                self.dat_paths
                    .get(&"additionalnpcgrpparts.dat".to_string())
                    .unwrap()
                    .path(),
            )?);

        let npc_name = wrap_into_id_map(deserialize_dat::<NpcNameDat>(
            self.dat_paths
                .get(&"npcname-ru.dat".to_string())
                .unwrap()
                .path(),
        )?);

        let mut mob_skill_anim = wrap_into_id_vec_map(deserialize_dat::<MobSkillAnimGrpDat>(
            self.dat_paths
                .get(&"mobskillanimgrp.dat".to_string())
                .unwrap()
                .path(),
        )?);

        let default_npc_name = NpcNameDat::default();

        {
            let mut types = HashMap::new();
            for v in &npc_grp {
                if let std::collections::hash_map::Entry::Vacant(e) = types.entry(v.npc_type) {
                    e.insert(v.id);
                }
            }
        }

        let mut warnings = vec![];

        for npc in npc_grp {
            let id = npc.id as u32;
            let npc_name_record = if let Some(v) = npc_name.get(&id) {
                v
            } else {
                warnings.push(Log {
                    level: LogLevel::Warning,
                    producer: "Npc Loader".to_string(),
                    log: format!(
                        "Npc[{}]: No record in npcname. Default will be used",
                        npc.id
                    ),
                });

                &default_npc_name
            };

            let mesh_params = WindowParams::new(NpcMeshParams {
                mesh: self.game_string_table.get_o(&npc.mesh).into(),
                textures: npc
                    .texture_1
                    .iter()
                    .map(|v| self.game_string_table.get_o(v).into())
                    .collect(),
                additional_textures: npc
                    .texture_2
                    .inner
                    .iter()
                    .map(|v| self.game_string_table.get_o(v).into())
                    .collect(),
                decorations: npc
                    .deco_effect
                    .iter()
                    .map(|v| NpcDecorationEffect {
                        effect: self.game_string_table.get_o(&v.effect).into(),
                        scale: v.scale,
                    })
                    .collect(),
                attack_effect: self.game_string_table.get_o(&npc.attack_effect).into(),
                speed: npc.npc_speed,
                draw_scale: npc.draw_scale,
                use_zoomincam: npc.use_zoom_in_cam,
                run_speed: npc.run_speed,
                walk_speed: npc.walk_speed,
                collision_radius_1: npc.collision.radius_1,
                collision_radius_2: npc.collision.radius_2,
                collision_height_1: npc.collision.height_1,
                collision_height_2: npc.collision.height_2,
            });

            let sound_params = WindowParams::new(NpcSoundParams {
                attack_sound: npc
                    .attack_sound
                    .iter()
                    .map(|v| self.game_string_table.get_o(v).into())
                    .collect(),
                defence_sound: npc
                    .defence_sound
                    .iter()
                    .map(|v| self.game_string_table.get_o(v).into())
                    .collect(),
                damage_sound: npc
                    .damage_sound
                    .iter()
                    .map(|v| self.game_string_table.get_o(v).into())
                    .collect(),
                dialog_sound: npc
                    .dialog_sounds
                    .inner
                    .iter()
                    .map(|v| self.game_string_table.get_o(v).into())
                    .collect(),
                vol: npc.sound_vol,
                rad: npc.sound_radius,
                random: npc.sound_random,
                priority: npc.sound_priority,
            });

            let summon_params = WindowParams::new(NpcSummonParams {
                summon_type: SummonType::from_u8(npc.summon_sort).unwrap(),
                max_count: npc.summon_max_count,
                grade: npc.summon_grade,
                silhouette: npc.silhouette,
            });

            let equipment = WindowParams::new(NpcEquipParams {
                left_hand: ItemId(npc.left_hand),
                right_hand: ItemId(npc.right_hand),
                chest: ItemId(npc.chest),
            });

            let skill_animations =
                WindowParams::new(if let Some(animations) = mob_skill_anim.remove(&id) {
                    animations
                        .iter()
                        .map(|v| NpcSkillAnimation {
                            id: SkillId(v.skill_id),
                            animation: self.game_string_table.get_o(&v.animation).into(),
                        })
                        .collect()
                } else {
                    vec![]
                });

            let additional_parts =
                WindowParams::new(npc_additional_parts_grp.remove(&id).map(|parts| {
                    NpcAdditionalParts {
                        class: self.game_string_table.get_o(&parts.class).into(),
                        chest: ItemId(parts.chest),
                        legs: ItemId(parts.legs),
                        gloves: ItemId(parts.gloves),
                        feet: ItemId(parts.feet),
                        back: ItemId(parts.back),
                        hair_accessory: ItemId(parts.hair_accessory),
                        hair_style: parts.hair_style,
                        right_hand: ItemId(parts.right_hand),
                        left_hand: ItemId(parts.left_hand),
                    }
                }));

            let quest_infos = npc
                .quests
                .iter()
                .map(|v| NpcQuestInfo {
                    id: QuestId(v.id as u32),
                    step: v.step,
                })
                .collect();

            let mut properties = vec![];
            let mut iterator = npc.properties.iter();

            while let Some(id) = iterator.next() {
                let Some(level) = iterator.next() else { break };

                if *id == 0 {
                    continue;
                }

                properties.push(NpcProperty {
                    id: SkillId(*id as u32),
                    level: *level,
                });
            }

            let npc = Npc {
                id: NpcId(id),
                name: npc_name_record.name.to_string(),
                title: npc_name_record.title.to_string(),
                title_color: Color32::from_rgba_unmultiplied(
                    npc_name_record.title_color.r,
                    npc_name_record.title_color.g,
                    npc_name_record.title_color.b,
                    npc_name_record.title_color.a,
                ),
                npc_type: npc.npc_type,
                unreal_script_class: self.game_string_table.get_o(&npc.unreal_class).into(),
                mesh_params,
                sound_params,
                summon_params,
                equipment_params: equipment,
                skill_animations,
                properties,
                social: npc.social == 1,
                show_hp: npc.show_hp == 1,
                org_hp: npc.hp,
                org_mp: npc.mp,
                icon: self.game_string_table.get_o(&npc.npc_icon).into(),
                additional_parts,
                quest_infos,
                ..Default::default()
            };

            self.npc_holder.insert(NpcId(id), npc);
        }

        Ok(warnings)
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct NpcNameDat {
    id: DWORD,
    name: ASCF,
    title: ASCF,
    title_color: Color,
}

impl Default for NpcNameDat {
    fn default() -> Self {
        Self {
            id: 0,
            name: ASCF::empty(),
            title: ASCF::empty(),
            title_color: Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
        }
    }
}

impl GetId for NpcNameDat {
    #[inline(always)]
    fn get_id(&self) -> u32 {
        self.id
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct DecoEffect {
    effect: DWORD,
    scale: FLOAT,
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct NpcQuestData {
    id: USHORT,
    step: BYTE,
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct NpcGrpDat {
    id: USHORT,
    unreal_class: DWORD,
    mesh: DWORD,
    texture_1: Vec<DWORD>,
    texture_2: UVEC<DWORD, DWORD>,
    properties: Vec<USHORT>,
    npc_speed: FLOAT,
    attack_sound: Vec<DWORD>,
    defence_sound: Vec<DWORD>,
    damage_sound: Vec<DWORD>,
    deco_effect: Vec<DecoEffect>,
    quests: Vec<NpcQuestData>,
    attack_effect: DWORD,
    sound_vol: BYTE,
    sound_radius: BYTE,
    sound_random: BYTE,
    social: BYTE,
    show_hp: BYTE,
    dialog_sounds: UVEC<DWORD, DWORD>,
    silhouette: BYTE,
    summon_sort: BYTE,
    summon_max_count: BYTE,
    summon_grade: BYTE,
    draw_scale: FLOAT,
    use_zoom_in_cam: FLOAT,
    npc_icon: DWORD,
    sound_priority: BYTE,
    run_speed: USHORT,
    walk_speed: USHORT,
    collision: Collision,
    left_hand: DWORD,
    right_hand: DWORD,
    chest: DWORD,
    hp: DOUBLE,
    mp: DOUBLE,
    npc_type: USHORT,
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct AdditionalNpcGrpPartsDat {
    npc_id: DWORD,
    class: DWORD,
    chest: DWORD,
    legs: DWORD,
    gloves: DWORD,
    feet: DWORD,
    back: DWORD,
    hair_accessory: DWORD,
    hair_style: DWORD,
    right_hand: DWORD,
    left_hand: DWORD,
}

impl GetId for AdditionalNpcGrpPartsDat {
    #[inline(always)]
    fn get_id(&self) -> u32 {
        self.npc_id
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct MobSkillAnimGrpDat {
    npc_id: DWORD,
    skill_id: DWORD,
    animation: DWORD,
}

impl GetId for MobSkillAnimGrpDat {
    #[inline(always)]
    fn get_id(&self) -> u32 {
        self.npc_id
    }
}
