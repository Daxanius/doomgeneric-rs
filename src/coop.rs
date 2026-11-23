use crate::game::{self, DoomGeneric, Pixel, DOOMGENERIC_RESX, DOOMGENERIC_RESY};
use crate::input::KeyData;
use std::fmt;
use std::marker::PhantomData;
use std::os::raw::{c_int, c_uint};
use std::sync::atomic::{AtomicBool, Ordering};

pub const MAX_PLAYERS: usize = 4;

pub mod buttons {
    pub const ATTACK: u8 = 1;
    pub const USE: u8 = 2;
    pub const CHANGE: u8 = 4;
    pub const WEAPON_SHIFT: u8 = 3;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TicCmd {
    pub forward_move: i8,
    pub side_move:    i8,
    pub angle_turn:  i16,
    pub chat_char: u8,
    pub buttons: u8,
    pub consistancy: u8,
    pub buttons2: u8,
    pub inventory: i32,
   
    // Heretic/Hexen specific:
    pub lookfly: u8, // look/fly up/down/centering
    pub arti: u8     // artitype_t to use
}

impl TicCmd {
    pub const IDLE: TicCmd = TicCmd {
        forward_move: 0,
        side_move: 0,
        angle_turn: 0,
        chat_char: 0,
        buttons: 0,
        consistancy: 0,
        buttons2: 0,
        inventory: 0,
        lookfly: 0,
        arti: 0
    };

    #[must_use]
    pub fn with_attack(mut self, on: bool) -> Self {
        if on {
            self.buttons |= buttons::ATTACK;
        }
        self
    }

    #[must_use]
    pub fn with_use(mut self, on: bool) -> Self {
        if on {
            self.buttons |= buttons::USE;
        }
        self
    }

    /// Select a weapon by its number key, 1 to 7
    #[must_use]
    pub fn with_weapon_slot(mut self, slot: u8) -> Self {
        if (1..=7).contains(&slot) {
            self.buttons |= buttons::CHANGE | ((slot - 1) << buttons::WEAPON_SHIFT);
        }
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Coop,
    Deathmatch,
    AltDeath,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub iwad: String,
    pub players: usize,
    pub mode: Mode,
    pub episode: u8,
    pub map: u8,
    pub skill: u8,
    pub no_monsters: bool,
    pub respawn_monsters: bool,
    pub fast_monsters: bool,
    pub time_limit_minutes: u32,
    pub seed: u8,
    pub extra_args: Vec<String>,
}

impl Config {
    pub fn new(iwad: impl Into<String>) -> Config {
        Config {
            iwad: iwad.into(),
            players: 2,
            mode: Mode::Deathmatch,
            episode: 1,
            map: 1,
            skill: 3,
            no_monsters: false,
            respawn_monsters: false,
            fast_monsters: false,
            time_limit_minutes: 0,
            seed: 0,
            extra_args: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    AlreadyCreated,
    InvalidPlayerCount(usize),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::AlreadyCreated => write!(
                f,
                "a Game was already created in this process (the engine cannot be restarted)"
            ),
            Error::InvalidPlayerCount(n) => {
                write!(f, "{n} players requested, but 1 to {MAX_PLAYERS} are supported")
            }
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Level,
    Intermission,
    Finale,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerState {
    pub alive: bool,
    pub health: i32,
    pub armor: i32,
    pub armor_type: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub angle: u32,
    pub mom_x: i32,
    pub mom_y: i32,
    pub mom_z: i32,
    pub ready_weapon: i32,
    pub ammo: [i32; 4],
    pub frags: [i32; MAX_PLAYERS],
}

impl PlayerState {
    pub fn position(&self) -> (f32, f32) {
        (self.x as f32 / 65536.0, self.y as f32 / 65536.0)
    }

    pub fn angle_degrees(&self) -> f32 {
        self.angle as f32 / 4_294_967_296.0 * 360.0
    }
}

/// One rendered view
/// Once more, the interface should be unified :P
pub struct Frame {
    pub pixels: Vec<Pixel>,
    #[cfg(feature = "cmap256")]
    pub palette: Box<[u32; 256]>,
}

impl Frame {
    pub const WIDTH: usize = DOOMGENERIC_RESX;
    pub const HEIGHT: usize = DOOMGENERIC_RESY;

    #[cfg(feature = "cmap256")]
    pub fn to_rgb32(&self) -> Vec<u32> {
        self.pixels
            .iter()
            .map(|&index| self.palette[index as usize] & 0x00FF_FFFF)
            .collect()
    }

    #[cfg(not(feature = "cmap256"))]
    pub fn to_rgb32(&self) -> Vec<u32> {
        self.pixels.iter().map(|&pixel| pixel & 0x00FF_FFFF).collect()
    }

    pub fn to_rgb8(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.pixels.len() * 3);
        for rgb in self.to_rgb32() {
            out.extend_from_slice(&[(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8]);
        }
        out
    }
}

#[repr(C)]
struct RawConfig {
    num_players: c_int,
    deathmatch: c_int,
    episode: c_int,
    map: c_int,
    skill: c_int,
    nomonsters: c_int,
    respawn_monsters: c_int,
    fast_monsters: c_int,
    timelimit: c_int,
    seed: c_int,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct RawPlayerState {
    in_game: c_int,
    alive: c_int,
    health: c_int,
    armor: c_int,
    armor_type: c_int,
    x: i32,
    y: i32,
    z: i32,
    angle: u32,
    momx: i32,
    momy: i32,
    momz: i32,
    ready_weapon: c_int,
    ammo: [c_int; 4],
    frags: [c_int; MAX_PLAYERS],
}

const _: () = assert!(std::mem::size_of::<TicCmd>() == 16);
const _: () = assert!(std::mem::size_of::<RawConfig>() == 40);
const _: () = assert!(std::mem::size_of::<RawPlayerState>() == 84);

extern "C" {
    fn dg_coop_configure(cfg: *const RawConfig);
    fn dg_coop_step(cmds: *const TicCmd, active_mask: c_uint) -> c_int;
    fn dg_coop_gametic() -> c_int;
    fn dg_coop_gamestate() -> c_int;
    fn dg_coop_player_state(player: c_int, out: *mut RawPlayerState) -> c_int;
    fn dg_coop_render_player(player: c_int, out: *mut Pixel) -> c_int;
}

// This is bad, I need to unify the interface
struct Headless;

impl DoomGeneric for Headless {
    fn draw_frame(&mut self, _screen_buffer: &[Pixel], _xres: usize, _yres: usize) {}

    fn get_key(&mut self) -> Option<KeyData> {
        None
    }

    fn get_mouse_delta(&mut self) -> i16 {
        0
    }

    fn set_window_title(&mut self, _title: &str) {}
}

static CREATED: AtomicBool = AtomicBool::new(false);

pub struct Game {
    players: usize,
    active: u8,
    _single_thread: PhantomData<*const ()>,
}

impl Game {
    pub fn new(config: &Config) -> Result<Game, Error> {
        if config.players == 0 || config.players > MAX_PLAYERS {
            return Err(Error::InvalidPlayerCount(config.players));
        }
        if CREATED.swap(true, Ordering::SeqCst) {
            return Err(Error::AlreadyCreated);
        }

        let raw_config = RawConfig {
            num_players: config.players as c_int,
            deathmatch: match config.mode {
                Mode::Coop => 0,
                Mode::Deathmatch => 1,
                Mode::AltDeath => 2,
            },
            episode: c_int::from(config.episode),
            map: c_int::from(config.map),
            skill: c_int::from(config.skill),
            nomonsters: c_int::from(config.no_monsters),
            respawn_monsters: c_int::from(config.respawn_monsters),
            fast_monsters: c_int::from(config.fast_monsters),
            timelimit: config.time_limit_minutes.cast_signed(),
            seed: c_int::from(config.seed),
        };

        let mut args = vec![
            "doomgeneric".to_string(),
            "-iwad".to_string(),
            config.iwad.clone(),
        ];
        args.extend(config.extra_args.iter().cloned());

        unsafe {
            // PREPARE FOR LIFTOOFFF
            dg_coop_configure(&raw const raw_config);
        }
        game::init(&args, Headless);

        Ok(Game {
            players: config.players,
            active: ((1u16 << config.players) - 1) as u8,
            _single_thread: PhantomData,
        })
    }

    pub fn players(&self) -> usize {
        self.players
    }

    pub fn step(&mut self, cmds: &[TicCmd]) -> u32 {
        let mut all = [TicCmd::IDLE; MAX_PLAYERS];
        for (dst, src) in all.iter_mut().zip(cmds.iter().take(self.players)) {
            *dst = *src;
        }
        let tic = unsafe { dg_coop_step(all.as_ptr(), c_uint::from(self.active)) };
        tic.max(0).cast_unsigned()
    }

    /// Current tic number.
    #[must_use]
    pub fn tic(&self) -> u32 {
        let tic = unsafe { dg_coop_gametic() };
        tic.max(0).cast_unsigned()
    }

    #[must_use]
    pub fn game_state(&self) -> GameState {
        match unsafe { dg_coop_gamestate() } {
            0 => GameState::Level,
            1 => GameState::Intermission,
            2 => GameState::Finale,
            _ => GameState::Other,
        }
    }

    pub fn remove_player(&mut self, player: usize) {
        if player < self.players {
            self.active &= !(1u8 << player);
        }
    }

    pub fn is_active(&self, player: usize) -> bool {
        player < self.players && self.active & (1u8 << player) != 0
    }

    #[must_use]
    pub fn player_state(&self, player: usize) -> Option<PlayerState> {
        if player >= MAX_PLAYERS {
            return None;
        }
        let mut raw = RawPlayerState::default();
        let found = unsafe { dg_coop_player_state(player as c_int, &raw mut raw) };
        if found == 0 {
            return None;
        }
        Some(PlayerState {
            alive: raw.alive != 0,
            health: raw.health,
            armor: raw.armor,
            armor_type: raw.armor_type,
            x: raw.x,
            y: raw.y,
            z: raw.z,
            angle: raw.angle,
            mom_x: raw.momx,
            mom_y: raw.momy,
            mom_z: raw.momz,
            ready_weapon: raw.ready_weapon,
            ammo: raw.ammo,
            frags: raw.frags,
        })
    }

    pub fn render_into(&mut self, player: usize, buffer: &mut [Pixel]) -> bool {
        assert_eq!(
            buffer.len(),
            DOOMGENERIC_RESX * DOOMGENERIC_RESY,
            "buffer must hold RESX * RESY pixels"
        );
        if player >= MAX_PLAYERS {
            return false;
        }
        let rendered = unsafe { dg_coop_render_player(player as c_int, buffer.as_mut_ptr()) };
        rendered != 0
    }

    pub fn render(&mut self, player: usize) -> Option<Frame> {
        let mut pixels = vec![0 as Pixel; DOOMGENERIC_RESX * DOOMGENERIC_RESY];
        if !self.render_into(player, &mut pixels) {
            return None;
        }
        Some(Frame {
            pixels,
            #[cfg(feature = "cmap256")]
            palette: Box::new(game::palette()),
        })
    }
}