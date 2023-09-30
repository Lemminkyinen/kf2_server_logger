-- Your SQL goes here

CREATE TABLE
  `players_` (
    `steam_id` bigint(20) NOT NULL,
    `name` varchar(50) NOT NULL,
    `count` mediumint(9) NOT NULL DEFAULT 1,
    `ip_address` int(10) unsigned DEFAULT NULL,
    `ping` mediumint(9) DEFAULT NULL,
    `unique_net_id` varchar(50) DEFAULT NULL,
    `last_joined` datetime NOT NULL DEFAULT current_timestamp() ON UPDATE current_timestamp(),
    PRIMARY KEY (`steam_id`)
  ) ENGINE = InnoDB DEFAULT CHARSET = latin1 COLLATE = latin1_swedish_ci