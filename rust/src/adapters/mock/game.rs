use tokio::sync::oneshot;

pub fn rust_frame() {
    unimplemented!();
}

pub struct Game {
    // Require that Game is only instantiated once through the new constructor
    #[allow(dead_code)]
    field: (),
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
