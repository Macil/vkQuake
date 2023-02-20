CREATE TABLE IF NOT EXISTS maps_with_context (
    id INTEGER PRIMARY KEY,
    map_name TEXT NOT NULL,
    game TEXT NOT NULL,
    map_display_name TEXT NOT NULL,
    secret_count INTEGER NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_maps_with_context__unique
ON maps_with_context (game, map_name);

CREATE TABLE IF NOT EXISTS secrets_found (
    map_context_id INTEGER NOT NULL,
    idx INTEGER NOT NULL,
    first_found_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (map_context_id, idx),
    FOREIGN KEY (map_context_id) REFERENCES maps_with_context(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS map_completions (
    id INTEGER PRIMARY KEY,
    map_context_id INTEGER NOT NULL,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    gametype TEXT NOT NULL,
    skill INTEGER NOT NULL,
    max_simultaneous_players INTEGER NOT NULL,
    cheats_used BOOLEAN NOT NULL CHECK (cheats_used IN (0, 1)),
    completed_time INTEGER NOT NULL,
    secrets_found_json TEXT NOT NULL,
    monsters_killed INTEGER NOT NULL,
    monsters_total INTEGER NOT NULL,
    FOREIGN KEY (map_context_id) REFERENCES maps_with_context(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_map_completions__by_timestamp
ON map_completions (timestamp);
CREATE INDEX IF NOT EXISTS idx_map_completions__by_map_timestamp
ON map_completions (map_context_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_map_completions__by_map_completed_time
ON map_completions (map_context_id, completed_time ASC);
CREATE INDEX IF NOT EXISTS idx_map_completions__by_map_monsters_killed
ON map_completions (map_context_id, monsters_killed DESC);
CREATE INDEX IF NOT EXISTS idx_map_completions__by_map_secrets_found
ON map_completions (map_context_id, json_array_length(secrets_found_json) DESC);
