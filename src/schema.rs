// @generated automatically by Diesel CLI.

diesel::table! {
    current_players (name) {
        #[max_length = 50]
        name -> Varchar,
        #[max_length = 50]
        perk -> Varchar,
        health -> Unsigned<Integer>,
        dosh -> Unsigned<Integer>,
        kills -> Unsigned<Integer>,
        ping -> Unsigned<Integer>,
    }
}

diesel::table! {
    game_sessions (id) {
        id -> Unsigned<Integer>,
        max_waves -> Unsigned<Smallint>,
        reached_wave -> Unsigned<Smallint>,
        max_players -> Unsigned<Smallint>,
        players_at_most -> Unsigned<Smallint>,
        #[max_length = 50]
        map_name -> Varchar,
        #[max_length = 50]
        difficulty -> Varchar,
        #[max_length = 50]
        game_type -> Varchar,
        #[max_length = 50]
        boss -> Varchar,
        started_at -> Timestamp,
        ended_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    ip_addresses (id) {
        id -> Unsigned<Integer>,
        steam_id -> Unsigned<Bigint>,
        ip_address -> Unsigned<Integer>,
        created -> Datetime,
    }
}

diesel::table! {
    player_sessions (id) {
        id -> Unsigned<Integer>,
        game_session_id -> Unsigned<Integer>,
        steam_id -> Unsigned<Bigint>,
        #[max_length = 50]
        perk -> Varchar,
        kills -> Unsigned<Integer>,
        started_at -> Timestamp,
        ended_at -> Timestamp,
    }
}

diesel::table! {
    unique_players (steam_id) {
        steam_id -> Unsigned<Bigint>,
        #[max_length = 50]
        name -> Varchar,
        maps_played -> Unsigned<Integer>,
        avg_ping -> Unsigned<Integer>,
        #[max_length = 50]
        unique_net_id -> Varchar,
        created -> Datetime,
        last_seen -> Datetime,
    }
}

diesel::joinable!(ip_addresses -> unique_players (steam_id));
diesel::joinable!(player_sessions -> game_sessions (game_session_id));
diesel::joinable!(player_sessions -> unique_players (steam_id));

diesel::allow_tables_to_appear_in_same_query!(
    current_players,
    game_sessions,
    ip_addresses,
    player_sessions,
    unique_players,
);
