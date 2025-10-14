use crate::backend::dat_loader::protocol_166::Color;
use crate::backend::dat_loader::GetId;
use l2_rw::ue2_rw::{ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal, ASCF, DWORD};
use r#macro::{ReadUnreal, WriteUnreal};

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct SysMessageDat {
    id: DWORD,
    unk_0: DWORD,
    message: ASCF,
    group: DWORD,
    color: Color,
    sound: DWORD,
    voice: DWORD,
    win: DWORD,
    font: DWORD,
    life_time: DWORD,
    bkg: DWORD,
    anim: DWORD,
    screen_msg: ASCF,
    screen_param: ASCF,
    gfx_screen_msg: ASCF,
    gfx_screen_param: ASCF,
    s_type: ASCF,
}

impl GetId for SysMessageDat {
    #[inline(always)]
    fn get_id(&self) -> u32 {
        self.id
    }
}
