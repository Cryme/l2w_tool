use crate::backend::log_holder::Log;

use l2_rw::ue2_rw::{ASCF, DWORD, INT};
use l2_rw::{DatVariant, deserialize_dat, save_dat};

use l2_rw::ue2_rw::{ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal};

use crate::backend::holder::{GameDataHolder, HolderMapOps};
use crate::common::AnimationComboId;
use crate::entity::animation_combo::AnimationCombo;
use r#macro::{ReadUnreal, WriteUnreal};
use std::thread;
use std::thread::JoinHandle;

impl GameDataHolder {
    pub fn serialize_animation_combo_to_binary(&mut self) -> JoinHandle<Log> {
        let raid_grp: Vec<AnimationComboDat> = self
            .animation_combo_holder
            .values()
            .filter(|v| !v._deleted)
            .map(|v| AnimationComboDat {
                name: self.game_string_table_ru.get_index(&v.name),
                anim_0: (&v.anim_0).into(),
                anim_1: (&v.anim_1).into(),
                anim_2: (&v.anim_2).into(),
                loop_p: if v.loop_p { 1 } else { -1 },
            })
            .collect();

        let dat_path = self
            .dat_paths
            .get(&"animationcombo.dat".to_string())
            .unwrap()
            .clone();

        thread::spawn(move || {
            if let Err(e) = save_dat(
                dat_path.path(),
                DatVariant::<(), AnimationComboDat>::Array(raid_grp.to_vec()),
            ) {
                Log::from_loader_e(e)
            } else {
                Log::from_loader_i("AnimationCombo saved")
            }
        })
    }

    pub fn load_animation_combo(&mut self) -> Result<Vec<Log>, ()> {
        let warnings = vec![];

        let animation_combos = deserialize_dat::<AnimationComboDat>(
            self.dat_paths
                .get(&"animationcombo.dat".to_string())
                .unwrap()
                .path(),
        )?;

        for (i, v) in animation_combos.iter().enumerate() {
            let name = self.game_string_table_ru.get_o(&v.name);

            let id = AnimationComboId(i as u32);
            self.animation_combo_holder.insert(
                id,
                AnimationCombo {
                    id,

                    name,
                    anim_0: v.anim_0.to_string(),
                    anim_1: v.anim_1.to_string(),
                    anim_2: v.anim_2.to_string(),
                    loop_p: v.loop_p == 1,

                    _changed: false,
                    _deleted: false,
                },
            );
        }

        Ok(warnings)
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct AnimationComboDat {
    name: DWORD,
    anim_0: ASCF,
    anim_1: ASCF,
    anim_2: ASCF,
    loop_p: INT,
}
