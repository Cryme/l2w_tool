use crate::backend::log_holder::Log;
use std::io::{Read, Write};

use l2_rw::ue2_rw::{ASCF, DWORD};
use l2_rw::{deserialize_dat, save_dat, DatVariant};

use l2_rw::ue2_rw::{ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal};

use crate::backend::dat_loader::GetId;
use crate::backend::holder::{GameDataHolder, HolderMapOps};
use crate::common::PlayerClass;
use crate::entity::daily_mission::{
    DailyMission, DailyMissionRepeatType, DailyMissionReward, DailyMissionUnk7,
};
use num_traits::{FromPrimitive, ToPrimitive};
use r#macro::{ReadUnreal, WriteUnreal};
use std::thread;
use std::thread::JoinHandle;

impl GameDataHolder {
    pub fn serialize_daily_missions_to_binary(&mut self) -> JoinHandle<Log> {
        let onedayrewards: Vec<OneDayRewardDat> = self
            .daily_mission_holder
            .values()
            .filter(|v| !v._deleted)
            .map(|v| OneDayRewardDat {
                base: OneDayRewardBase {
                    id: v.id.0,
                    reward_id: v.reward_id,
                    reward_name: ASCF::from(&v.name),
                    rewards_ct: v.rewards.len() as u32,
                    reward_desc: ASCF::from(&v.desc),
                    reward_period: ASCF::from(&v.category),
                    allowed_classes: if let Some(c) = &v.allowed_classes {
                        c.iter().map(|v| v.to_u32().unwrap()).collect()
                    } else {
                        vec![u32::MAX]
                    },
                    repeat_type: v.repeat_type.to_u32().unwrap(),
                    unk2: v.unk2,
                    unk3: v.unk3,
                    unk4: v.unk4,
                    unk5: v.unk5,
                    unk6: v.unk6,
                    unk7_count: v.unk7.len() as u32,
                    unk8: v.unk8.clone(),
                    category: v.category_type,
                },
                unk7: v
                    .unk7
                    .iter()
                    .map(|c| OneDayRewardUnk7 {
                        unk1: c.unk1,
                        unk2: c.unk2,
                        unk3: c.unk3,
                        unk4: c.unk4,
                    })
                    .collect(),
                rewards: v
                    .rewards
                    .iter()
                    .map(|c| OneDayRewardsInfo {
                        item_id: c.item_id.0,
                        count: c.count,
                    })
                    .collect(),
            })
            .collect();

        let dat_path = self
            .dat_paths
            .get(&"onedayreward-ru.dat".to_string())
            .unwrap()
            .clone();

        thread::spawn(move || {
            if let Err(e) = save_dat(
                dat_path.path(),
                DatVariant::<(), OneDayRewardDat>::Array(onedayrewards.to_vec()),
            ) {
                Log::from_loader_e(e)
            } else {
                Log::from_loader_i("DailyMissions saved")
            }
        })
    }

    pub fn load_daily_missions(&mut self) -> Result<Vec<Log>, ()> {
        let one_day_rewards = deserialize_dat::<OneDayRewardDat>(
            self.dat_paths
                .get(&"onedayreward-ru.dat".to_string())
                .unwrap()
                .path(),
        )?;

        for v in one_day_rewards {
            self.daily_mission_holder.insert(
                v.base.id.into(),
                DailyMission {
                    id: v.base.id.into(),
                    reward_id: v.base.reward_id,
                    name: v.base.reward_name.to_string(),
                    desc: v.base.reward_desc.to_string(),
                    category: v.base.reward_period.to_string(),
                    category_type: v.base.category,
                    allowed_classes: if v.base.allowed_classes.is_empty()
                        || v.base.allowed_classes[0] == u32::MAX
                    {
                        None
                    } else {
                        Some(
                            v.base
                                .allowed_classes
                                .iter()
                                .map(|c| {
                                    PlayerClass::from_u32(*c).unwrap_or_else(|| panic!("!!UNK {c}"))
                                })
                                .collect(),
                        )
                    },
                    repeat_type: DailyMissionRepeatType::from_u32(v.base.repeat_type).unwrap(),
                    unk2: v.base.unk2,
                    unk3: v.base.unk3,
                    unk4: v.base.unk4,
                    unk5: v.base.unk5,
                    unk6: v.base.unk6,
                    unk7: v
                        .unk7
                        .iter()
                        .map(|c| DailyMissionUnk7 {
                            unk1: c.unk1,
                            unk2: c.unk2,
                            unk3: c.unk3,
                            unk4: c.unk4,
                        })
                        .collect(),
                    unk8: v.base.unk8,
                    rewards: v
                        .rewards
                        .iter()
                        .map(|c| DailyMissionReward {
                            item_id: c.item_id.into(),
                            count: c.count,
                        })
                        .collect(),

                    _changed: false,
                    _deleted: false,
                },
            );
        }

        Ok(vec![])
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct OneDayRewardUnk7 {
    unk1: DWORD,
    unk2: DWORD,
    unk3: DWORD,
    unk4: DWORD,
}
#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct OneDayRewardsInfo {
    item_id: DWORD,
    count: DWORD,
}

#[derive(Debug, Clone, PartialEq)]
struct OneDayRewardDat {
    base: OneDayRewardBase,
    unk7: Vec<OneDayRewardUnk7>,
    rewards: Vec<OneDayRewardsInfo>,
}

impl ReadUnreal for OneDayRewardDat {
    fn read_unreal<T: Read>(reader: &mut T) -> Self {
        let base: OneDayRewardBase = reader.read_unreal_value();

        let mut unk7 = vec![];
        for _ in 0..base.unk7_count {
            unk7.push(reader.read_unreal_value())
        }

        let mut rewards = vec![];
        for _ in 0..base.rewards_ct {
            rewards.push(reader.read_unreal_value())
        }

        Self {
            base,
            unk7,
            rewards,
        }
    }
}

impl WriteUnreal for OneDayRewardDat {
    fn write_unreal<T: Write>(&self, writer: &mut T) -> std::io::Result<()> {
        writer.write_unreal_value(&self.base)?;

        for v in &self.unk7 {
            writer.write_unreal_value(v)?;
        }
        for v in &self.rewards {
            writer.write_unreal_value(v)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct OneDayRewardBase {
    id: DWORD,
    reward_id: DWORD,
    reward_name: ASCF,
    rewards_ct: DWORD,
    reward_desc: ASCF,
    reward_period: ASCF,
    allowed_classes: Vec<DWORD>,
    repeat_type: DWORD,
    unk2: DWORD,
    unk3: DWORD,
    unk4: DWORD,
    unk5: DWORD,
    unk6: DWORD,
    unk7_count: DWORD,
    unk8: Vec<DWORD>,
    category: DWORD,
}

impl GetId for OneDayRewardDat {
    #[inline(always)]
    fn get_id(&self) -> u32 {
        self.base.id
    }
}
