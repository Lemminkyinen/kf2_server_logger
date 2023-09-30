use crate::kf2_scrape::models::PlayerInfo;
use chrono;
use diesel::prelude::*;
use diesel_async::AsyncMysqlConnection;
use std::net::Ipv4Addr;

#[derive(Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::players_)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct PlayerDb {
    pub(super) steam_id: u64,
    pub(super) name: String,
    pub(super) count: u32,
    pub(super) ip_address: u32,
    pub(super) ping: u32,
    pub(super) unique_net_id: String,
    pub(super) last_joined: chrono::NaiveDateTime,
}

impl PlayerDb {
    pub fn from(player_steam: PlayerInfo) -> Self {
        let ip_address = match player_steam.ip {
            std::net::IpAddr::V4(ip) => ip,
            _ => Ipv4Addr::new(0, 0, 0, 0),
        };
        let ip_address: u32 = ip_address.into();
        let last_joined = chrono::Local::now().naive_utc();
        Self {
            steam_id: player_steam.steam_id,
            name: player_steam.name,
            count: 0,
            ip_address,
            ping: player_steam.ping,
            unique_net_id: player_steam.unique_net_id,
            last_joined,
        }
    }
}
