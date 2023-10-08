-- Your SQL goes here
CREATE TABLE player_sessions (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    game_session_id INT UNSIGNED NOT NULL,
    steam_id BIGINT UNSIGNED NOT NULL,
    perk VARCHAR(50) NOT NULL,
    kills INT UNSIGNED NOT NULL,
    started_at TIMESTAMP NOT NULL,
    ended_at TIMESTAMP NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (game_session_id) REFERENCES game_sessions(id),
    FOREIGN KEY (steam_id) REFERENCES unique_players(steam_id)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE utf8mb4_swedish_ci;