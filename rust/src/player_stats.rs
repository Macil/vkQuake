use crate::{adapters::game::ClGame, RECORD_PLAYER_STATS, RUNTIME};

// TODO record secrets found too

pub fn level_completed(cl_game: &mut ClGame) {
    fn attempt(cl_game: &mut ClGame) -> anyhow::Result<()> {
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
        let secrets_found = cl_game.secrets_found();
        let monsters_killed = cl_game.monsters_killed()?;
        let monsters_total = cl_game.monsters_total()?;
        let gametype = cl_game.gametype().to_string();

        // TODO this should be the highest number of players that were in the game at any point
        let max_simultaneous_players = cl_game.player_count();

        // TODO
        let cheats_used = false;

        RUNTIME.spawn_blocking(move || {
            let mut db = crate::DB.get().unwrap().lock().unwrap();

            db.insert_map_completion(
                &map_name,
                &game,
                &map_display_name,
                map_secret_count,
                &gametype,
                max_simultaneous_players,
                cheats_used,
                completed_time,
                &secrets_found,
                monsters_killed,
                monsters_total,
            )
            .unwrap();
        });
        Ok(())
    }

    if let Err(err) = attempt(cl_game) {
        tracing::error!("player_stats level_completed: {}", err);
    }
}
