#![forbid(unsafe_code)]
#![allow(dead_code)]
#![allow(unused_variables)]

use std::path::PathBuf;

pub mod console;
pub mod cvar;
pub mod game;

pub fn game_pref_path() -> PathBuf {
    unimplemented!();
}
