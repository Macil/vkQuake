#![deny(unsafe_op_in_unsafe_fn)]

mod adapters;
mod axum_helpers;
mod middleware;
mod tracing_init;

use adapters::{
    cvar::{Cvar, CvarFlags},
    game::GameInit,
};
use axum::{
    extract,
    routing::{get, patch},
    Json, Router,
};
use axum_helpers::QuakeAPIResponseError;
use hyperlocal_with_windows::{remove_unix_socket_if_present, UnixServerExt};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::runtime::Runtime;
use tower::ServiceBuilder;
use tower_http::{trace::TraceLayer, validate_request::ValidateRequestHeaderLayer};

/// Should only need to be used directly by synchronous code called from C.
pub(crate) static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().unwrap());

pub(crate) fn init(game_init: &mut GameInit) {
    // TODO
    // add the following cvars:
    // - remote_api_http_enabled, default 1
    // - remote_api_http_ip, default "all"
    // - remote_api_http_port, default 27060
    // - remote_api_allow_signaling_server, default 1
    // - remote_api_signaling_servers, default "https://quakesignal.macil.tech"

    // add commands:
    // - remote_api_pair (and remote_api_pair_confirm, remote_api_pair_cancel)
    // - remote_api_add_http_user user passwordhash level
    // - remote_api_list_users
    // - remote_api_remove_user user

    // TODO always listen on Unix socket
    // TODO only listen on TCP if users list is nonempty.

    // The list of users (including permission levels, password hashes, user agent string, last seen) is stored in
    // vkQuake pref directory / remote_api_users.toml.
    // [users.user1]
    // password_hash = "$123"
    // permission_level = "game" // (game, admin, none)
    // last_seen = 2023-01-01T12:00:00Z

    // TODO support cvars and commands that can't be executed on client
    // by maps (PR_stuffcmd) or server packet or demos (SVC_STUFFTEXT). It looks like Cmd_ExecuteString's
    // cmd_source parameter contains some logic like this for console commands already.
    // I'm just not sure if it distinguishes commands from maps vs the player and what it allows specifically.
    // Also look at how Host_Savegame_f checks cmd_source.

    let remote_api_http_enabled = Cvar::register(
        game_init,
        "remote_api_http_enabled",
        "1",
        CvarFlags::CVAR_ARCHIVE,
    );

    Cvar::load_early(game_init, &[remote_api_http_enabled.name()]).unwrap();

    // TODO react to cvar changes
    let remote_api_http_enabled_value = remote_api_http_enabled.value(&game_init.game);
    RUNTIME.spawn(async move {
        tokio::spawn(async move {
            if let Err(e) = listen_on_unix_socket().await {
                tracing::error!("Error listening on unix socket: {}", e);
            }
        });

        if remote_api_http_enabled_value == 1.0 {
            if let Err(e) = listen_on_tcp_socket().await {
                tracing::error!("Error listening on tcp socket: {}", e);
            }
        }
    });
}

async fn listen_on_unix_socket() -> anyhow::Result<()> {
    let unix_socket_path = adapters::game_pref_path().join("remote_api.sock");
    remove_unix_socket_if_present(&unix_socket_path).await?;
    let app = create_app(true);
    tracing::info!("listening on unix socket");
    axum::Server::bind_unix(&unix_socket_path)?
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn listen_on_tcp_socket() -> anyhow::Result<()> {
    let app = create_app(false);
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::try_bind(&addr)?
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

fn create_app(is_unix_socket: bool) -> Router {
    let mut app = Router::new()
        .route("/", get(root))
        .route("/entities", get(entities))
        .route("/player", get(player))
        .route("/player", patch(patch_player))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(ValidateRequestHeaderLayer::custom(
                    middleware::csrf_protection::CsrfProtection::new(),
                )),
        );
    if !is_unix_socket {
        app = app.layer(ValidateRequestHeaderLayer::custom(
            middleware::rebinding_protection::RebindingProtection::new(vec![
                "localhost".to_string()
            ]),
        ))
    }
    app
}

async fn root() -> &'static str {
    "Hello, World!"
}

#[derive(Serialize, Debug, Clone, PartialEq)]
struct PlayerEntity {
    health: f32,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
struct PatchPlayerEntity {
    health: Option<f32>,
}

async fn player() -> Result<Json<Option<PlayerEntity>>, QuakeAPIResponseError> {
    let player_health =
        adapters::game::Game::run_in_game_thread_with_result(|game| game.player_health()).await?;
    let value = player_health.map(|health| PlayerEntity { health });
    Ok(Json(value))
}

async fn patch_player(
    extract::Json(patch): extract::Json<PatchPlayerEntity>,
) -> Result<Json<()>, QuakeAPIResponseError> {
    adapters::game::Game::run_in_game_thread_mut(move |game| {
        tracing::info!("updating player: {:?}", patch);
        if let Some(health) = patch.health {
            game.set_player_health(health);
        }
    })
    .await;
    Ok(Json(()))
}

async fn entities() -> &'static str {
    "TODO all entities in JSON form"
}
