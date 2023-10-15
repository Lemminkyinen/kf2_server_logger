-- Your SQL goes here
CREATE TABLE 
    `current_players` (
        `name` VARCHAR(50) NOT NULL,
        `perk` VARCHAR(50) NOT NULL,
        `health` INT UNSIGNED NOT NULL,
        `dosh` INT UNSIGNED NOT NULL,
        `kills` INT UNSIGNED NOT NULL,
        `ping` INT UNSIGNED NOT NULL,     
        PRIMARY KEY (`name`)   
    ) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE utf8mb4_swedish_ci;