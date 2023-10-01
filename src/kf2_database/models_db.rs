use crate::kf2_scrape::models::{Perk, PlayerInGame, PlayerInfo};
use chrono;
use diesel::prelude::*;
use log::error;
use std::net::Ipv4Addr;

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::ip_addresses)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct IpAddressDbQ {
    pub(super) id: u32,
    pub(super) steam_id: u64,
    pub(super) ip_address: u32,
    pub(super) created: chrono::NaiveDateTime,
}
#[derive(Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::ip_addresses)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct IpAddressDbI {
    pub(super) steam_id: u64,
    pub(super) ip_address: u32,
}

impl IpAddressDbI {
    pub fn from(player_steam: PlayerInfo) -> Self {
        let ip_address = match player_steam.ip {
            std::net::IpAddr::V4(ip) => ip,
            ip => {
                error!("Invalid IP address {}", ip);
                Ipv4Addr::new(0, 0, 0, 0)
            }
        };
        Self {
            steam_id: player_steam.steam_id,
            ip_address: ip_address.into(),
        }
    }
}

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::unique_players)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct PlayerDbQ {
    pub(super) steam_id: u64,
    pub(super) name: String,
    pub(super) maps_played: u32,
    pub(super) avg_ping: u32,
    pub(super) unique_net_id: String,
    pub(super) created: chrono::NaiveDateTime,
    pub(super) last_seen: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::unique_players)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub(crate) struct PlayerDbI {
    pub(super) steam_id: u64,
    pub(super) name: String,
    pub(super) maps_played: u32,
    pub(super) avg_ping: u32,
    pub(super) unique_net_id: String,
    pub(super) last_seen: chrono::NaiveDateTime,
}

impl From<PlayerInfo> for PlayerDbI {
    fn from(player_steam: PlayerInfo) -> Self {
        let last_joined = chrono::Local::now().naive_utc();
        Self {
            steam_id: player_steam.steam_id,
            name: player_steam.name,
            maps_played: 0,
            avg_ping: player_steam.ping,
            unique_net_id: player_steam.unique_net_id,
            last_seen: last_joined,
        }
    }
}

#[derive(Insertable, Clone)]
#[diesel(table_name = crate::schema::current_players)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub(super) struct CurrentPlayer {
    name: String,
    perk: String,
    health: u32,
    dosh: u32,
    kills: u32,
    ping: u32,
}

impl From<PlayerInGame> for CurrentPlayer {
    fn from(player: PlayerInGame) -> Self {
        Self {
            name: player.name,
            perk: player.perk.to_string(),
            health: player.health,
            dosh: player.dosh,
            kills: player.kills,
            ping: player.ping,
        }
    }
}
