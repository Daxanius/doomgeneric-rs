use std::os::raw;

use crate::game::{DoomGeneric, DOOM_HANDLER};

pub const MAX_PLAYERS: usize = 4;

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct DoomInputPacketRaw {
    pub forwardmove: raw::c_char, // signed char
    pub sidemove: raw::c_char,    // signed char
    pub angleturn: raw::c_short,  // short
    pub chatchar: raw::c_char,    // byte
    pub buttons: raw::c_char,     // byte
    pub consistancy: raw::c_char, // byte

    // Strife specific
    pub buttons2: raw::c_char, // byte
    pub inventory: raw::c_int, // int

    // Heretic/Hexen specific
    pub lookfly: raw::c_char, // byte
    pub arti: raw::c_char,    // byte
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct DoomGameSettingsRaw {
    pub ticdup: raw::c_int,
    pub extratics: raw::c_int,
    pub deathmatch: raw::c_int,
    pub episode: raw::c_int,
    pub nomonsters: raw::c_int,
    pub fast_monsters: raw::c_int,
    pub respawn_monsters: raw::c_int,
    pub map: raw::c_int,
    pub skill: raw::c_int,
    pub gameversion: raw::c_int,
    pub lowres_turn: raw::c_int,
    pub new_sync: raw::c_int,
    pub timelimit: raw::c_int,
    pub loadgame: raw::c_int,
    pub random: raw::c_int,
    // Start message fields
    pub num_players: raw::c_int,
    pub consoleplayer: raw::c_int,
    // Hexen classes (ensure NET_MAXPLAYERS matches C, usually 4 or 8)
    pub player_classes: [raw::c_int; 8],
}

extern "C" {
    pub fn DG_CL_SetCmdBundle(cmds: *const DoomInputPacketRaw, player_mask: *const raw::c_int);

    pub fn DG_CL_SetLocalPlayer(player_num: raw::c_int);

    pub fn DG_CL_SpawnPlayer(player_num: raw::c_int);

    pub fn DG_CL_RemovePlayer(player_num: raw::c_int);

    pub fn DG_CL_RemoveAllPlayers();
}

#[no_mangle]
pub unsafe extern "C" fn DG_CL_GetSettings(settings: *mut DoomGameSettingsRaw) {
    if settings.is_null() {
        return;
    }

    if let Some(doom_box) = unsafe { DOOM_HANDLER.get_mut() }.as_mut() {
        doom_box.get_settings(unsafe { &mut *settings });
    }
}

#[no_mangle]
pub unsafe extern "C" fn DG_CL_SendTiccmd(
    cmd: *const DoomInputPacketRaw,
    maketic: raw::c_int,
    player_id: raw::c_int,
) {
    if cmd.is_null() {
        return;
    }

    if let Some(doom_box) = unsafe { DOOM_HANDLER.get_mut() }.as_mut() {
        doom_box.send_tic_cmd(unsafe { &*cmd }, maketic, player_id);
    }
}
