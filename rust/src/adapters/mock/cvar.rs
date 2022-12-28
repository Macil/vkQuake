use super::game::{Game, GameInit};

pub use super::super::common::CvarFlags;

#[derive(Debug)]
pub struct Cvar(());

impl Cvar {
    pub fn register(
        _game_init: &mut GameInit,
        name: &str,
        default: &str,
        flags: CvarFlags,
    ) -> Cvar {
        // noop
        Cvar(())
    }

    /// Must be used if you want to read the value of a cvar before the game is initialized.
    pub fn load_early(_game_init: &mut GameInit, names: &[&str]) -> anyhow::Result<()> {
        unimplemented!();
    }

    pub fn find_existing_by_name(_game: &Game, name: &str) -> anyhow::Result<Cvar> {
        unimplemented!();
    }

    pub fn name(&self) -> &str {
        unimplemented!();
    }

    pub fn str_value<'a>(&self, _game: &'a Game) -> Result<Option<&'a str>, std::str::Utf8Error> {
        unimplemented!();
    }

    pub fn value(&self, _game: &Game) -> f32 {
        unimplemented!();
    }

    pub fn set_value(&self, _game: &mut Game, value: f32) {
        unimplemented!();
    }

    pub fn set_str_value(&self, _game: &mut Game, value: &str) {
        unimplemented!();
    }
}
