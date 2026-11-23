use doomgeneric::coop::{self, buttons, Config, GameState, Mode, TicCmd};
use doomgeneric::game::{self, DoomGeneric, Pixel, DOOMGENERIC_RESX, DOOMGENERIC_RESY};
use doomgeneric::input::{keys, KeyData};
use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
use std::collections::{HashSet, VecDeque};
use std::time::{Duration, Instant};

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

// MULTIPLAYER
const TIC_RATE: u64 = 35;

fn parse_number<'a, T, I>(it: &mut I, flag: &str) -> T
where
    T: std::str::FromStr,
    I: Iterator<Item = &'a String>,
{
    it.next()
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(|| panic!("{} needs a number", flag))
}

fn parse_coop_args(args: &[String]) -> Config {
    let mut config = Config::new("");
    let mut iwad: Option<String> = None;

    let mut it = args.iter().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--players" => config.players = parse_number(&mut it, "--players"),
            "--coop" => config.mode = Mode::Coop,
            "--dm" => config.mode = Mode::Deathmatch,
            "--seed" => config.seed = parse_number(&mut it, "--seed"),
            "--skill" => config.skill = parse_number(&mut it, "--skill"),
            "--episode" => config.episode = parse_number(&mut it, "--episode"),
            "--map" => config.map = parse_number(&mut it, "--map"),
            "-iwad" => iwad = it.next().cloned(),
            other => config.extra_args.push(other.to_string()),
        }
    }

    config.iwad = iwad.expect("-iwad <path to WAD> is required");
    config
}

/// Keyboard input for player 1
fn read_input(window: &Window) -> TicCmd {
    let down = |key: Key| window.is_key_down(key);
    let run = down(Key::LeftShift) || down(Key::RightShift);
    let walk: i8 = if run { 50 } else { 25 };
    let turn: i16 = if run { 1280 } else { 640 };

    let mut cmd = TicCmd::IDLE;
    if down(Key::Up) || down(Key::W) {
        cmd.forward_move += walk;
    }
    if down(Key::Down) || down(Key::S) {
        cmd.forward_move -= walk;
    }
    if down(Key::D) {
        cmd.side_move += walk;
    }
    if down(Key::A) {
        cmd.side_move -= walk;
    }
    if down(Key::Left) || down(Key::Q) {
        cmd.angle_turn += turn;
    }
    if down(Key::Right) || down(Key::E) {
        cmd.angle_turn -= turn;
    }
    if down(Key::LeftCtrl) || down(Key::RightCtrl) || down(Key::F) {
        cmd.buttons |= buttons::ATTACK;
    }
    if down(Key::Space) {
        cmd.buttons |= buttons::USE;
    }

    let weapon_keys = [
        Key::Key1, Key::Key2, Key::Key3, Key::Key4, Key::Key5, Key::Key6, Key::Key7,
    ];
    for (slot, key) in weapon_keys.iter().enumerate() {
        if down(*key) {
            cmd = cmd.with_weapon_slot(slot as u8 + 1);
        }
    }
    cmd
}

/// A very dumb bot
fn dumb_bot(player: usize, tic: u32) -> TicCmd {
    let mut cmd = TicCmd {
        forward_move: 50,
        side_move: 0,
        angle_turn: if (tic as usize / 40 + player) % 2 == 1 { 640 } else { -320 },
        buttons: 0,
        ..Default::default()
    };
    if tic % 8 < 2 {
        cmd.buttons |= buttons::ATTACK;
    }
    if tic % 17 == 0 {
        cmd.buttons |= buttons::USE;
    }
    cmd
}

fn window_title(view: usize, players: usize) -> String {
    format!(
        "DOOM co-op - viewing player {}/{}",
        view + 1,
        players
    )
}

fn run_coop(config: Config) {
    let players = config.players;
    let mut game = coop::Game::new(&config).expect("failed to start the game");

    let mut window = Window::new(
        &window_title(0, players),
        DOOMGENERIC_RESX,
        DOOMGENERIC_RESY,
        WindowOptions {
            scale: Scale::X2,
            ..WindowOptions::default()
        },
    )
    .expect("failed to create window");

    let tic_duration = Duration::from_micros(1_000_000 / TIC_RATE);
    let mut next_tic = Instant::now();

    let mut rgb = vec![0u32; DOOMGENERIC_RESX * DOOMGENERIC_RESY];
    let mut view = 0usize;
    let mut tab_was_down = false;
    let mut tic = 0u32;

    while window.is_open() {
        let tab_down = window.is_key_down(Key::Tab);
        if tab_down && !tab_was_down {
            view = (view + 1) % players;
            window.set_title(&window_title(view, players));
        }
        tab_was_down = tab_down;

        let mut cmds = vec![TicCmd::IDLE; players];
        cmds[0] = read_input(&window);
        for player in 1..players {
            cmds[player] = dumb_bot(player, tic);
        }

        game.step(&cmds);
        tic += 1;

        if let Some(frame) = game.render(view) {
            rgb = frame.to_rgb32();
        }
        let _ = window.update_with_buffer(&rgb, DOOMGENERIC_RESX, DOOMGENERIC_RESY);

        next_tic += tic_duration;
        let now = Instant::now();
        if next_tic > now {
            std::thread::sleep(next_tic - now);
        } else {
            next_tic = now;
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|arg| arg == "--players") {
        run_coop(parse_coop_args(&args));
        return;
    }

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