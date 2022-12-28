use tokio::sync::oneshot;

pub struct Game {
    // Require that Game is only instantiated once through the new constructor
    _field: (),
}

impl Game {
    pub async fn run_in_game_thread(f: impl FnOnce(&Game) + Send + 'static) {
        unimplemented!();
    }

    pub async fn run_in_game_thread_with_result<R: Send + 'static>(
        f: impl FnOnce(&Game) -> R + Send + 'static,
    ) -> Result<R, oneshot::error::RecvError> {
        unimplemented!();
    }

    pub async fn run_in_game_thread_mut(f: impl FnOnce(&mut Game) + Send + 'static) {
        unimplemented!();
    }

    pub async fn run_in_game_thread_mut_with_result<R: Send + 'static>(
        f: impl FnOnce(&mut Game) -> R + Send + 'static,
    ) -> Result<R, oneshot::error::RecvError> {
        unimplemented!();
    }

    pub fn player_health(&self) -> Option<f32> {
        None
    }

    pub fn set_player_health(&mut self, health: f32) {}

    pub fn rcon(&mut self, cmd: &str) {
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
fn dummy_rust_init() {
    let mut game_init = GameInit {
        game: Game { _field: () },
        _field: (),
    };

    crate::init(&mut game_init);
}
