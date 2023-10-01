-- Your SQL goes here
CREATE TABLE
  `unique_players` (
    `steam_id` BIGINT UNSIGNED NOT NULL,
    `name` VARCHAR(50) NOT NULL,
    `maps_played` INT UNSIGNED NOT NULL DEFAULT 1,
    `avg_ping` INT UNSIGNED NOT NULL,
    `unique_net_id` VARCHAR(50) NOT NULL,
    `created` datetime NOT NULL DEFAULT current_timestamp(),
    `last_seen` datetime NOT NULL DEFAULT current_timestamp() ON UPDATE current_timestamp(),
    PRIMARY KEY (`steam_id`)
  ) ENGINE = InnoDB DEFAULT CHARSET = latin1 COLLATE = latin1_swedish_ci;

CREATE TABLE
  `ip_addresses` (
    `id` INT UNSIGNED NOT NULL AUTO_INCREMENT,
    `steam_id` BIGINT UNSIGNED NOT NULL,
    `ip_address` INT UNSIGNED NOT NULL,
    `created` datetime NOT NULL DEFAULT current_timestamp(),
    PRIMARY KEY (`id`),
    FOREIGN KEY (`steam_id`) REFERENCES `unique_players` (`steam_id`)
  ) ENGINE = InnoDB DEFAULT CHARSET = latin1 COLLATE = latin1_swedish_ci;