-- Your SQL goes here
CREATE TABLE 
    `current_players` (
        `id` INT UNSIGNED NOT NULL AUTO_INCREMENT,
        `name` VARCHAR(50) NOT NULL,
        `perk` VARCHAR(50) NOT NULL,
        `health` INT UNSIGNED NOT NULL,
        `dosh` INT UNSIGNED NOT NULL,
        `kills` INT UNSIGNED NOT NULL,
        `ping` INT UNSIGNED NOT NULL,     
        PRIMARY KEY (`id`)   
    ) ENGINE = InnoDB DEFAULT CHARSET = latin1 COLLATE = latin1_swedish_ci;