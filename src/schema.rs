diesel::table! {
    players_ (steam_id) {
        steam_id -> Unsigned<BigInt>,
        name -> Text,
        count -> Unsigned<Integer>,
        ip_address -> Unsigned<Integer>,
        ping -> Unsigned<Integer>,
        unique_net_id -> Text,
        last_joined -> Datetime,
    }
}
