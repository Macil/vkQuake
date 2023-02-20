use std::{num::TryFromIntError, str::Utf8Error};

use tokio::sync::oneshot;

use crate::adapters::common::Gametype;

pub struct Game {
    // Require that Game is only instantiated once through the new constructor
    _field: (),
}

impl Game {
    pub fn game_names(&self) -> Result<String, Utf8Error> {
        unimplemented!();
    }
}

pub struct SvGame {
    pub game: Game,
    // Require that SvGame is only instantiated within this module.
    _field: (),
}

impl SvGame {
    pub async fn run_in_game_thread(f: impl FnOnce(&Self) + Send + 'static) {
        unimplemented!();
    }

    pub async fn run_in_game_thread_with_result<R: Send + 'static>(
        f: impl FnOnce(&Self) -> R + Send + 'static,
    ) -> Result<R, oneshot::error::RecvError> {
        unimplemented!();
    }

    pub async fn run_in_game_thread_mut(f: impl FnOnce(&mut Self) + Send + 'static) {
        unimplemented!();
    }

    pub async fn run_in_game_thread_mut_with_result<R: Send + 'static>(
        f: impl FnOnce(&mut Self) -> R + Send + 'static,
    ) -> Result<R, oneshot::error::RecvError> {
        unimplemented!();
    }

    pub fn player_health(&self) -> Option<f32> {
        None
    }

    pub fn set_player_health(&mut self, health: f32) {}

    pub fn rcon(&mut self, cmd: &str) -> String {
        unimplemented!();
    }
}

pub struct ClGame {
    pub game: Game,
    // Require that ClGame is only instantiated within this module.
    _field: (),
}

impl ClGame {
    pub fn demo_playback(&self) -> bool {
        unimplemented!();
    }
    pub fn map_name(&self) -> Result<&str, Utf8Error> {
        unimplemented!();
    }
    pub fn map_display_name(&self) -> Result<&str, Utf8Error> {
        unimplemented!();
    }
    pub fn map_secret_count(&self) -> Result<u32, TryFromIntError> {
        unimplemented!();
    }
    pub fn completed_time(&self) -> Result<u32, TryFromIntError> {
        unimplemented!();
    }
    pub fn secrets_found(&self) -> Vec<u32> {
        unimplemented!();
    }
    pub fn monsters_killed(&self) -> Result<u32, TryFromIntError> {
        unimplemented!();
    }
    pub fn monsters_total(&self) -> Result<u32, TryFromIntError> {
        unimplemented!();
    }
    pub fn player_count(&self) -> u32 {
        unimplemented!();
    }
    pub fn gametype(&self) -> Gametype {
        unimplemented!();
    }
}

pub struct GameInit {
    pub game: Game,
    // Require that Game is only instantiated within this module.
    _field: (),
}

// This function exists just so functions like crate::init don't show as unused functions
// in test builds.
fn dummy_rust_calls() {
    let mut game_init = GameInit {
        game: Game { _field: () },
        _field: (),
    };
    crate::init(&mut game_init);

    let mut cl_game = ClGame {
        game: Game { _field: () },
        _field: (),
    };
    crate::player_stats::level_completed(&mut cl_game);
}
