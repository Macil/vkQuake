use super::raw_bindings::{
    cl, cls, cmd_source_t_src_command, svs, COM_GetGameNames, Cmd_ExecuteString, Con_Redirect,
    STAT_MONSTERS, STAT_TOTALMONSTERS, STAT_TOTALSECRETS,
};
use crate::adapters::common::Gametype;
use crate::tracing_init;
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::convert::{TryFrom, TryInto};
use std::ffi::CString;
use std::num::TryFromIntError;
use std::str::Utf8Error;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::{SendError, TryRecvError};
use tokio::sync::oneshot;

type HostFrameCallback = Box<dyn FnOnce(&mut SvGame) + Send + 'static>;

static mut HOST_FRAME_QUEUE: Lazy<(
    mpsc::Sender<HostFrameCallback>,
    mpsc::Receiver<HostFrameCallback>,
)> = Lazy::new(|| mpsc::channel(200));
static HOST_FRAME_QUEUE_TX: Lazy<&mpsc::Sender<HostFrameCallback>> =
    Lazy::new(|| unsafe { &HOST_FRAME_QUEUE.0 });

thread_local! {
    static REDIRECTED_CONSOLE_OUTPUT: RefCell<Vec<String>> = RefCell::default();
}

#[no_mangle]
pub unsafe extern "C" fn Rust_Frame() {
    let mut sv_game = SvGame {
        game: Game { _field: () },
        _field: (),
    };
    let rx = unsafe { &mut HOST_FRAME_QUEUE.1 };
    loop {
        match rx.try_recv() {
            Ok(f) => f(&mut sv_game),
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => panic!("FRAME_QUEUE disconnected"),
        }
    }
}

pub struct Game {
    // Require that Game is only instantiated within this module.
    _field: (),
}

impl Game {
    pub fn game_names(&self) -> Result<String, Utf8Error> {
        let s = unsafe { std::ffi::CStr::from_ptr(COM_GetGameNames(1)) };
        s.to_str().map(|s| s.to_owned())
    }
}

pub struct SvGame {
    pub game: Game,
    // Require that SvGame is only instantiated within this module.
    _field: (),
}

impl SvGame {
    pub async fn run_in_game_thread(f: impl FnOnce(&Self) + Send + 'static) {
        // TODO allow multiple run_in_game_thread callbacks to run concurrently
        HOST_FRAME_QUEUE_TX
            .send(Box::new(move |sv_game| {
                f(sv_game);
            }))
            .await
            // Replace the error with something that doesn't reference the callback
            // type, so that the error implements Debug.
            .map_err(|_| SendError(()))
            .unwrap();
    }

    pub async fn run_in_game_thread_with_result<R: Send + 'static>(
        f: impl FnOnce(&Self) -> R + Send + 'static,
    ) -> Result<R, oneshot::error::RecvError> {
        let (tx, rx) = oneshot::channel();
        Self::run_in_game_thread(move |sv_game| {
            let _ = tx.send(f(sv_game));
        })
        .await;
        rx.await
    }

    #[allow(dead_code)]
    pub async fn run_in_game_thread_mut(f: impl FnOnce(&mut Self) + Send + 'static) {
        HOST_FRAME_QUEUE_TX
            .send(Box::new(f))
            .await
            // Replace the error with something that doesn't reference the callback
            // type, so that the error implements Debug.
            .map_err(|_| SendError(()))
            .unwrap();
    }

    #[allow(dead_code)]
    pub async fn run_in_game_thread_mut_with_result<R: Send + 'static>(
        f: impl FnOnce(&mut Self) -> R + Send + 'static,
    ) -> Result<R, oneshot::error::RecvError> {
        let (tx, rx) = oneshot::channel();
        Self::run_in_game_thread_mut(|sv_game| {
            let _ = tx.send(f(sv_game));
        })
        .await;
        rx.await
    }

    fn clients(&self) -> &[super::raw_bindings::client_s] {
        unsafe { std::slice::from_raw_parts(svs.clients, svs.maxclients.try_into().unwrap()) }
    }

    fn clients_mut(&mut self) -> &mut [super::raw_bindings::client_s] {
        unsafe { std::slice::from_raw_parts_mut(svs.clients, svs.maxclients.try_into().unwrap()) }
    }

    /// Use this instead of checking `sv_player`, because `sv_player` often contains
    /// stale values!
    fn first_player(&self) -> Option<*const super::raw_bindings::edict_t> {
        for client in self.clients() {
            if client.active != 0 {
                return Some(client.edict);
            }
        }
        None
    }

    fn first_player_mut(&mut self) -> Option<*mut super::raw_bindings::edict_t> {
        for client in self.clients_mut() {
            if client.active != 0 {
                return Some(client.edict);
            }
        }
        None
    }

    pub fn player_health(&self) -> Option<f32> {
        let player = self.first_player();
        player.map(|p| unsafe { (*p).v.health })
    }

    pub fn set_player_health(&mut self, health: f32) {
        // TODO fix overhealth not draining, and fix <=0 values not killing the player.
        if let Some(p) = self.first_player_mut() {
            unsafe {
                (*p).v.health = health;
            }
        }
    }

    // TODO create rcon_tab_autocomplete function that returns a list of possible completions
    pub fn rcon(&mut self, cmd: &str) -> String {
        unsafe extern "C" fn rcon_redirect(s: *const std::ffi::c_char) {
            let s = unsafe { std::ffi::CStr::from_ptr(s) };
            REDIRECTED_CONSOLE_OUTPUT.with(|r| {
                r.borrow_mut().push(s.to_string_lossy().into_owned());
            });
        }

        assert!(REDIRECTED_CONSOLE_OUTPUT.with(|r| r.borrow().is_empty()));

        let cmd = CString::new(cmd).unwrap();
        unsafe {
            Con_Redirect(Some(rcon_redirect));
            Cmd_ExecuteString(cmd.as_ptr(), cmd_source_t_src_command);
            Con_Redirect(None);
        }
        REDIRECTED_CONSOLE_OUTPUT.with(|r| r.take()).concat()
    }
}

pub struct ClGame {
    pub game: Game,
    // Require that ClGame is only instantiated within this module.
    _field: (),
}

impl ClGame {
    pub fn demo_playback(&self) -> bool {
        unsafe { cls.demoplayback != 0 }
    }
    pub fn map_name(&self) -> Result<&str, Utf8Error> {
        unsafe { std::ffi::CStr::from_ptr(&cl.mapname[0]) }.to_str()
    }
    pub fn map_display_name(&self) -> Result<&str, Utf8Error> {
        unsafe { std::ffi::CStr::from_ptr(&cl.levelname[0]) }.to_str()
    }
    pub fn map_secret_count(&self) -> Result<u32, TryFromIntError> {
        u32::try_from(unsafe { cl.stats[STAT_TOTALSECRETS as usize] })
    }
    pub fn completed_time(&self) -> Result<u32, TryFromIntError> {
        u32::try_from(unsafe { cl.completed_time })
    }
    pub fn secrets_found(&self) -> Vec<u32> {
        // TODO
        Vec::new()
    }
    pub fn monsters_killed(&self) -> Result<u32, TryFromIntError> {
        u32::try_from(unsafe { cl.stats[STAT_MONSTERS as usize] })
    }
    pub fn monsters_total(&self) -> Result<u32, TryFromIntError> {
        u32::try_from(unsafe { cl.stats[STAT_TOTALMONSTERS as usize] })
    }
    pub fn player_count(&self) -> u32 {
        let scores =
            unsafe { std::slice::from_raw_parts(cl.scores, cl.maxclients.try_into().unwrap()) };
        scores
            .iter()
            .filter(|s| s.name[0] != 0)
            .count()
            .try_into()
            .unwrap()
    }
    pub fn gametype(&self) -> Gametype {
        if unsafe { cl.gametype == 0 } {
            if unsafe { cl.maxclients == 1 } {
                Gametype::Singleplayer
            } else {
                Gametype::Coop
            }
        } else {
            Gametype::Deathmatch
        }
    }
}

pub struct GameInit {
    pub game: Game,
    // Require that GameInit is only instantiated within this module.
    _field: (),
}

/// Sets up things that could come to be depended on by C code that's called earlier than Rust_Init.
#[no_mangle]
pub unsafe extern "C" fn Rust_Init_Early() {
    tracing_init::init();
}

// Disabled in tests because the rest of the crate expects mock Game references instead of real ones.
#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn Rust_Init() {
    let mut game_init = GameInit {
        game: Game { _field: () },
        _field: (),
    };

    crate::init(&mut game_init);
}

// Disabled in tests because the rest of the crate expects mock Game references instead of real ones.
#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn CL_Rust_Level_Completed() {
    let mut cl_game = ClGame {
        game: Game { _field: () },
        _field: (),
    };

    crate::player_stats::level_completed(&mut cl_game);
}
