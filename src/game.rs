use crate::client::DoomGameSettingsRaw;
use crate::client::DoomInputPacketRaw;
use crate::input::KeyData;
use std::cell::RefCell;
use std::convert::TryFrom;
use std::ffi::c_char;
use std::ffi::c_int;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw;
use std::sync::LazyLock;
use std::thread::sleep;
use std::time::{Duration, Instant};

/// The resolution is hardcoded in the underlying library as macro definitions.
pub const DOOMGENERIC_RESX: usize = 320;
pub const DOOMGENERIC_RESY: usize = 200;

static mut C_ARGS: Option<Vec<*const i8>> = None;

pub trait DoomGeneric {
    fn draw_frame(&mut self, screen_buffer: &[u8], xres: usize, yres: usize);
    fn get_key(&mut self) -> Option<KeyData>;
    fn get_mouse_delta(&mut self) -> i16;
    fn set_window_title(&mut self, title: &str);

    // Networking stuff
    /// Fills the settings struct. Returns true (1) if successful.
    fn get_settings(&mut self, settings: &mut DoomGameSettingsRaw);

    /// Sends the local ticcmd to the Minecraft relay.
    fn send_tic_cmd(&mut self, cmd: &DoomInputPacketRaw, maketic: i32, player_id: i32);
}

extern "C" {
    fn D_DoomMain(); // doomgeneric.h
    fn doomgeneric_Tick(); // doomgeneric.h
    fn M_FindResponseFile(); // used in main of i_main.c
    pub static mut myargc: raw::c_int;
    pub static mut myargv: *mut *mut raw::c_char;
}

#[no_mangle]
pub(crate) static mut DG_ScreenBuffer: *const u8 = std::ptr::null();
pub(crate) static mut SCREEN_BUFFER: RefCell<Option<Box<[u8]>>> = RefCell::new(None);
pub(crate) static mut DOOM_HANDLER: RefCell<Option<Box<dyn DoomGeneric>>> = RefCell::new(None);
pub(crate) static START_TIME: LazyLock<Instant> = LazyLock::new(Instant::now);

#[no_mangle]
extern "C" fn DG_Init() {
    unsafe {
        *SCREEN_BUFFER.get_mut() =
            Some(vec![0u8; DOOMGENERIC_RESX * DOOMGENERIC_RESY].into_boxed_slice());
        // Setting DG_ScreenBuffer to where the new buffer is
        DG_ScreenBuffer = SCREEN_BUFFER.get_mut().as_ref().unwrap().as_ptr();
    }
}

#[no_mangle]
extern "C" fn DG_GetKey(pressed: *mut raw::c_int, key: *mut raw::c_uchar) -> raw::c_int {
    if let Some(doom_box) = unsafe { DOOM_HANDLER.get_mut().as_mut() } {
        if let Some(keydata) = doom_box.get_key() {
            unsafe {
                // Not tested yet!
                *pressed = i32::from(keydata.pressed);
                *key = keydata.key;
            }
            1
        } else {
            0
        }
    } else {
        0
    }
}

#[no_mangle]
extern "C" fn DG_GetTicksMs() -> u32 {
    u32::try_from(START_TIME.elapsed().as_millis())
        .expect("Can't fit passed milliseconds into u32!")
}

#[no_mangle]
extern "C" fn DG_SleepMs(ms: u32) {
    sleep(Duration::from_millis(u64::from(ms)));
}

#[no_mangle]
extern "C" fn DG_DrawFrame() {
    if let Some(doom_box) = unsafe { DOOM_HANDLER.get_mut() }.as_mut() {
        if let Some(screen_buffer) = unsafe { SCREEN_BUFFER.get_mut() }.as_mut() {
            doom_box.draw_frame(screen_buffer, DOOMGENERIC_RESX, DOOMGENERIC_RESY);
        }
    }
}

#[no_mangle]
extern "C" fn DG_SetWindowTitle(title: *const raw::c_char) {
    let title = unsafe { CStr::from_ptr(title) }
        .to_str()
        .expect("Can't convert title c string to rust string");
    if let Some(doom_box) = unsafe { DOOM_HANDLER.get_mut() }.as_mut() {
        doom_box.set_window_title(title);
    }
}

#[no_mangle]
extern "C" fn DG_GetMouseDelta() -> i16 {
    if let Some(doom_box) = unsafe { DOOM_HANDLER.get_mut().as_mut() } {
        doom_box.get_mouse_delta()
    } else {
        0
    }
}

pub fn init(doom_impl: impl DoomGeneric + 'static, args: &Vec<String>) {
    unsafe {
        *DOOM_HANDLER.get_mut() = Some(Box::new(doom_impl));

        let args = std::env::args()
            .map(|arg| CString::new(arg).unwrap())
            .collect::<Vec<CString>>();

        let c_args = args
            .iter()
            .map(|arg| arg.as_ptr())
            .collect::<Vec<*const c_char>>();

        myargc = c_args.len() as c_int;
        myargv = c_args.as_ptr() as *mut *mut i8;

        C_ARGS = Some(c_args);

        M_FindResponseFile();
        DG_Init();
        D_DoomMain();
    }
}

pub fn tick() {
    unsafe {
        doomgeneric_Tick();
    }
}
