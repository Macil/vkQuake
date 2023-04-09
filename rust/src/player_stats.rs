use crate::{adapters::game::ClGame, RECORD_PLAYER_STATS, RUNTIME};

pub fn player_found_secret(cl_game: &mut ClGame, secret: u16) {
    (|| {
        if cl_game.demo_playback() || RECORD_PLAYER_STATS.get().unwrap().value(&cl_game.game) == 0.0
        {
            return Ok(());
        }

        tracing::debug!("player_stats player_found_secret");

        let map_name = cl_game.map_name()?.to_owned();
        let game = cl_game.game.game_names()?;
        let map_display_name = cl_game.map_display_name()?.to_owned();
        let map_secret_count = cl_game.map_secret_count()?;

        RUNTIME
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .spawn_blocking(move || {
                (|| {
                    let mut db_guard = crate::DB.lock().unwrap();
                    let db = db_guard.as_mut().unwrap();

                    db.insert_secret_found(
                        &map_name,
                        &game,
                        &map_display_name,
                        map_secret_count,
                        secret,
                    )?;
                    Ok(())
                })()
                .unwrap_or_else(|err: anyhow::Error| {
                    tracing::error!("player_stats player_found_secret: {}", err);
                });
            });
        Ok(())
    })()
    .unwrap_or_else(|err: anyhow::Error| {
        tracing::error!("player_stats player_found_secret: {}", err);
    });
}

pub fn level_completed(cl_game: &mut ClGame, skill: u16, secrets: &[u16]) {
    (|| {
        if cl_game.demo_playback() || RECORD_PLAYER_STATS.get().unwrap().value(&cl_game.game) == 0.0
        {
            return Ok(());
        }

        tracing::debug!("player_stats level_completed");

        let map_name = cl_game.map_name()?.to_owned();
        let game = cl_game.game.game_names()?;
        let map_display_name = cl_game.map_display_name()?.to_owned();
        let map_secret_count = cl_game.map_secret_count()?;
        let completed_time = cl_game.completed_time()?;
        let secrets = secrets.to_owned();
        let monsters_killed = cl_game.monsters_killed()?;
        let monsters_total = cl_game.monsters_total()?;
        let gametype = cl_game.gametype().to_string();

        // TODO this should be the highest number of players that were in the game at any point
        let max_simultaneous_players = cl_game.player_count();

        // TODO
        let cheats_used = false;

        RUNTIME
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .spawn_blocking(move || {
                (|| {
                    let mut db_guard = crate::DB.lock().unwrap();
                    let db = db_guard.as_mut().unwrap();

                    db.insert_map_completion(
                        &map_name,
                        &game,
                        &map_display_name,
                        map_secret_count,
                        &gametype,
                        skill,
                        max_simultaneous_players,
                        cheats_used,
                        completed_time,
                        &secrets,
                        monsters_killed,
                        monsters_total,
                    )?;
                    Ok(())
                })()
                .unwrap_or_else(|err: anyhow::Error| {
                    tracing::error!("player_stats level_completed: {}", err);
                });
            });
        Ok(())
    })()
    .unwrap_or_else(|err: anyhow::Error| {
        tracing::error!("player_stats level_completed: {}", err);
    });
}
