// @generated automatically by Diesel CLI.

diesel::table! {
    ip_addresses (id) {
        id -> Unsigned<Integer>,
        steam_id -> Unsigned<Bigint>,
        ip_address -> Unsigned<Integer>,
        created -> Datetime,
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

diesel::allow_tables_to_appear_in_same_query!(
    ip_addresses,
    unique_players,
);
