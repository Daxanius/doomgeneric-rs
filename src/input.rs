/// Information about a key getting pressed or released
#[derive(Debug, Clone, Copy)]
pub struct KeyData {
    pub pressed: bool,
    pub key: i32,
}

mod key_bindings {
    use std::os::raw;

    extern "C" {
        pub static key_right: raw::c_int;
        pub static key_left: raw::c_int;
        pub static key_up: raw::c_int;
        pub static key_down: raw::c_int;
        pub static key_strafeleft: raw::c_int;
        pub static key_straferight: raw::c_int;
        pub static key_fire: raw::c_int;
        pub static key_use: raw::c_int;
        pub static key_strafe: raw::c_int;
        pub static key_speed: raw::c_int;
    }
}

/// Common keys used in doom
/// Keys are based on keyboard keycodes. So extra keys, like letters should be available when provided the correct keycode.
/// ASCII chars for example can just be cast to a u8 to work.
pub mod keys {
    use super::key_bindings;
    use std::sync::LazyLock;

    pub static KEY_RIGHT: LazyLock<i32> = LazyLock::new(|| unsafe { key_bindings::key_right });
    pub static KEY_LEFT: LazyLock<i32> = LazyLock::new(|| unsafe { key_bindings::key_left });
    pub static KEY_UP: LazyLock<i32> = LazyLock::new(|| unsafe { key_bindings::key_up });
    pub static KEY_DOWN: LazyLock<i32> = LazyLock::new(|| unsafe { key_bindings::key_down });
    pub static KEY_STRAFELEFT: LazyLock<i32> =
        LazyLock::new(|| unsafe { key_bindings::key_strafeleft });
    pub static KEY_STRAFERIGHT: LazyLock<i32> =
        LazyLock::new(|| unsafe { key_bindings::key_straferight });
    pub static KEY_FIRE: LazyLock<i32> = LazyLock::new(|| unsafe { key_bindings::key_fire });
    pub static KEY_USE: LazyLock<i32> = LazyLock::new(|| unsafe { key_bindings::key_use });
    // When pressed, KEY_LEFT and KEY_RIGHT act like KEY_STRAFELEFT and KEY_STRAFERIGHT accordingly
    pub static KEY_STRAFE: LazyLock<i32> = LazyLock::new(|| unsafe { key_bindings::key_strafe });
    pub static KEY_SPEED: LazyLock<i32> = LazyLock::new(|| unsafe { key_bindings::key_speed });
    pub static KEY_ESCAPE: u8 = 27;
    pub static KEY_ENTER: u8 = b'\r';

    #[must_use]
    pub fn from_char(ascii_char: char) -> Option<u8> {
        if ascii_char.is_ascii() {
            Some(ascii_char as u8)
        } else {
            None
        }
    }
}
