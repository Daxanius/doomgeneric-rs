use doomgeneric::game::{self, DoomGeneric, Pixel, DOOMGENERIC_RESX, DOOMGENERIC_RESY};
use doomgeneric::input::{keys, KeyData};
use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
use std::collections::{HashSet, VecDeque};

const ASCII_KEYS: &[(Key, u8)] = &[
    (Key::A, b'a'), (Key::B, b'b'), (Key::C, b'c'), (Key::D, b'd'), (Key::E, b'e'),
    (Key::F, b'f'), (Key::G, b'g'), (Key::H, b'h'), (Key::I, b'i'), (Key::J, b'j'),
    (Key::K, b'k'), (Key::L, b'l'), (Key::M, b'm'), (Key::N, b'n'), (Key::O, b'o'),
    (Key::P, b'p'), (Key::Q, b'q'), (Key::R, b'r'), (Key::S, b's'), (Key::T, b't'),
    (Key::U, b'u'), (Key::V, b'v'), (Key::W, b'w'), (Key::X, b'x'), (Key::Y, b'y'),
    (Key::Z, b'z'),
    (Key::Key0, b'0'), (Key::Key1, b'1'), (Key::Key2, b'2'), (Key::Key3, b'3'),
    (Key::Key4, b'4'), (Key::Key5, b'5'), (Key::Key6, b'6'), (Key::Key7, b'7'),
    (Key::Key8, b'8'), (Key::Key9, b'9'),
    (Key::Minus, b'-'), (Key::Equal, b'='),
];

fn to_doom_key(key: Key) -> Option<u8> {
    let mapped = match key {
        Key::Left => *keys::KEY_LEFT,
        Key::Right => *keys::KEY_RIGHT,
        Key::Up => *keys::KEY_UP,
        Key::Down => *keys::KEY_DOWN,
        Key::LeftCtrl | Key::RightCtrl => *keys::KEY_FIRE,
        Key::Space => *keys::KEY_USE,
        Key::LeftShift | Key::RightShift => *keys::KEY_SPEED,
        Key::LeftAlt | Key::RightAlt => *keys::KEY_STRAFE,
        Key::Comma => *keys::KEY_STRAFELEFT,
        Key::Period => *keys::KEY_STRAFERIGHT,
        Key::Enter => keys::KEY_ENTER,
        Key::Escape => keys::KEY_ESCAPE,
        Key::Tab => 9,
        Key::Backspace => 0x7f,
        other => return ASCII_KEYS.iter().find(|(k, _)| *k == other).map(|(_, c)| *c),
    };
    Some(mapped)
}

struct Viewer {
    window: Window,
    rgb: Vec<u32>,
    key_queue: VecDeque<KeyData>,
    held: HashSet<u8>
}

impl Viewer {
    #[cfg(feature = "cmap256")]
    fn convert(&mut self, src: &[Pixel]) {
        let palette = game::palette();
        for (dst, &idx) in self.rgb.iter_mut().zip(src) {
            *dst = palette[idx as usize] & 0x00FF_FFFF;
        }
    }

    #[cfg(not(feature = "cmap256"))]
    fn convert(&mut self, src: &[Pixel]) {
        for (dst, &px) in self.rgb.iter_mut().zip(src) {
            *dst = px & 0x00FF_FFFF;
        }
    }

    fn collect_keys(&mut self) {
        let now: HashSet<u8> = if self.window.is_active() {
            self.window.get_keys().into_iter().filter_map(to_doom_key).collect()
        } else {
            HashSet::new()
        };
        for &key in now.difference(&self.held) {
            self.key_queue.push_back(KeyData { pressed: true, key });
        }
        for &key in self.held.difference(&now) {
            self.key_queue.push_back(KeyData { pressed: false, key });
        }
        self.held = now;
    }
}

impl DoomGeneric for Viewer {
    fn draw_frame(&mut self, screen_buffer: &[Pixel], xres: usize, yres: usize) {
        self.convert(screen_buffer);
        let _ = self.window.update_with_buffer(&self.rgb, xres, yres);

        if !self.window.is_open() {
            std::process::exit(0);
        }
        
        self.collect_keys();
    }

    fn get_key(&mut self) -> Option<KeyData> {
        self.key_queue.pop_front()
    }

    fn get_mouse_delta(&mut self) -> i16 {
        0
    }

    fn set_window_title(&mut self, title: &str) {
        self.window.set_title(title);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let window = Window::new(
        "DOOM",
        DOOMGENERIC_RESX,
        DOOMGENERIC_RESY,
        WindowOptions {
            scale: Scale::X2,
            ..WindowOptions::default()
        },
    )
    .expect("failed to create window");

    let viewer = Viewer {
        window,
        rgb: vec![0; DOOMGENERIC_RESX * DOOMGENERIC_RESY],
        key_queue: VecDeque::new(),
        held: HashSet::new(),
    };

    game::init(&args, viewer);
    loop {
        game::tick();
    }
}