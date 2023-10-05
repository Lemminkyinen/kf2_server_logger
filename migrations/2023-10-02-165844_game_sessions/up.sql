-- Your SQL goes here
CREATE TABLE game_sessions (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    max_waves SMALLINT UNSIGNED NOT NULL,
    reached_wave SMALLINT UNSIGNED NOT NULL,
    max_players SMALLINT UNSIGNED NOT NULL,
    players_at_most SMALLINT UNSIGNED NOT NULL,
    map_name VARCHAR(50) NOT NULL,
    difficulty VARCHAR(50) NOT NULL,
    game_type VARCHAR(50) NOT NULL,
    boss VARCHAR(50) NOT NULL,
    started_at TIMESTAMP NOT NULL,
    ended_at TIMESTAMP NULL,
    PRIMARY KEY (id)
) ENGINE = InnoDB DEFAULT CHARSET = utf16 COLLATE utf16_swedish_ci;