use super::raw_bindings::{cmd_source_t_src_command, svs, Cmd_ExecuteString};
use once_cell::sync::Lazy;
use std::ffi::CString;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::{SendError, TryRecvError};
use tokio::sync::oneshot;

pub type FrameCallback = Box<dyn FnOnce() + Send + 'static>;

static mut FRAME_QUEUE: Lazy<(mpsc::Sender<FrameCallback>, mpsc::Receiver<FrameCallback>)> =
    Lazy::new(|| mpsc::channel(200));
pub static FRAME_QUEUE_TX: Lazy<&mpsc::Sender<FrameCallback>> =
    Lazy::new(|| unsafe { &FRAME_QUEUE.0 });

/// # Safety
/// Must only be called on the main game thread.
pub unsafe fn rust_frame() {
    let rx = unsafe { &mut FRAME_QUEUE.1 };
    loop {
        match rx.try_recv() {
            Ok(f) => f(),
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => panic!("FRAME_QUEUE disconnected"),
        }
    }
}

pub struct Game {
    // Require that Game is only instantiated within this module.
    #[allow(dead_code)]
    field: (),
}

impl Game {
    pub async fn run_in_game_thread(f: impl FnOnce(&Game) + Send + 'static) {
        // TODO allow multiple run_in_game_thread callbacks to run concurrently
        let game = Game { field: () };
        FRAME_QUEUE_TX
            .send(Box::new(move || {
                f(&game);
            }))
            .await
            // Replace the error with something that doesn't reference the callback
            // type, so that the error implements Debug.
            .map_err(|_| SendError(()))
            .unwrap();
    }

    pub async fn run_in_game_thread_with_result<R: Send + 'static>(
        f: impl FnOnce(&Game) -> R + Send + 'static,
    ) -> Result<R, oneshot::error::RecvError> {
        let (tx, rx) = oneshot::channel();
        Self::run_in_game_thread(|game| {
            let _ = tx.send(f(game));
        })
        .await;
        rx.await
    }

    #[allow(dead_code)]
    pub async fn run_in_game_thread_mut(f: impl FnOnce(&mut Game) + Send + 'static) {
        let mut game = Game { field: () };
        FRAME_QUEUE_TX
            .send(Box::new(move || {
                f(&mut game);
            }))
            .await
            // Replace the error with something that doesn't reference the callback
            // type, so that the error implements Debug.
            .map_err(|_| SendError(()))
            .unwrap();
    }

    #[allow(dead_code)]
    pub async fn run_in_game_thread_mut_with_result<R: Send + 'static>(
        f: impl FnOnce(&mut Game) -> R + Send + 'static,
    ) -> Result<R, oneshot::error::RecvError> {
        let (tx, rx) = oneshot::channel();
        Self::run_in_game_thread_mut(|game| {
            let _ = tx.send(f(game));
        })
        .await;
        rx.await
    }

    fn clients(&self) -> &[super::raw_bindings::client_s] {
        unsafe { std::slice::from_raw_parts(svs.clients, svs.maxclients as usize) }
    }

    fn clients_mut(&mut self) -> &mut [super::raw_bindings::client_s] {
        unsafe { std::slice::from_raw_parts_mut(svs.clients, svs.maxclients as usize) }
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

    #[allow(dead_code)]
    pub fn rcon(&mut self, cmd: &str) {
        let cmd = CString::new(cmd).unwrap();
        unsafe {
            Cmd_ExecuteString(cmd.as_ptr(), cmd_source_t_src_command);
        }
    }
}
