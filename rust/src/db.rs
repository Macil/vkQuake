use rusqlite::{params, Connection, OptionalExtension};

use crate::adapters::game_pref_path;

#[derive(Debug)]
pub struct QuakeDb {
    conn: Connection,
}

impl QuakeDb {
    pub fn new() -> rusqlite::Result<Self> {
        let db_path = game_pref_path().join("main.db");
        let conn = Connection::open(db_path)?;

        let init_query = include_str!("db/init.sql");
        conn.execute_batch(init_query)?;

        Ok(Self { conn })
    }

    pub fn get_or_insert_map_with_context(
        &mut self,
        map_name: &str,
        game: &str,
        map_display_name: &str,
        secret_count: u32,
    ) -> rusqlite::Result<i64> {
        let id = self
            .conn
            .query_row(
                "SELECT id FROM maps_with_context WHERE map_name = ? AND game = ?",
                params![map_name, game],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(id) = id {
            return Ok(id);
        }
        self.conn.execute(
            "INSERT OR IGNORE INTO maps_with_context (map_name, game, map_display_name, secret_count) VALUES (?, ?, ?, ?)",
            params![map_name, game, map_display_name, secret_count],
        )?;
        self.conn.query_row(
            "SELECT id FROM maps_with_context WHERE map_name = ? AND game = ?",
            params![map_name, game],
            |row| row.get(0),
        )
    }

    /// Records that a player found a specific secret at this current time.
    /// Does nothing if the secret has already been found.
    pub fn insert_secret_found(
        &mut self,
        map_name: &str,
        game: &str,
        map_display_name: &str,
        map_secret_count: u32,
        secret: u16,
    ) -> rusqlite::Result<()> {
        let map_context_id = self.get_or_insert_map_with_context(
            map_name,
            game,
            map_display_name,
            map_secret_count,
        )?;
        self.conn.execute(
            "INSERT OR IGNORE INTO secrets_found (map_context_id, idx, first_found_timestamp) VALUES (?, ?, CURRENT_TIMESTAMP)",
            params![map_context_id, secret],
        )?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn insert_map_completion(
        &mut self,
        map_name: &str,
        game: &str,
        map_display_name: &str,
        map_secret_count: u32,
        gametype: &str,
        skill: u16,
        max_simultaneous_players: u32,
        cheats_used: bool,
        completed_time: u32,
        secrets_found: &[u16],
        monsters_killed: u32,
        monsters_total: u32,
    ) -> rusqlite::Result<()> {
        let secrets_found_json = serde_json::to_string(&secrets_found).unwrap();
        let map_context_id = self.get_or_insert_map_with_context(
            map_name,
            game,
            map_display_name,
            map_secret_count,
        )?;
        self.conn.execute(
            "INSERT INTO map_completions (map_context_id, timestamp, gametype, skill, max_simultaneous_players, cheats_used, completed_time, secrets_found_json, monsters_killed, monsters_total) VALUES (?, CURRENT_TIMESTAMP, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![map_context_id, gametype, skill, max_simultaneous_players, cheats_used, completed_time, secrets_found_json, monsters_killed, monsters_total],
        )?;
        Ok(())
    }
}
