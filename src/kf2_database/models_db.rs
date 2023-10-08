use crate::{
    kf2_log::logger::{GameSession, PlayerSession},
    kf2_scrape::models::{KfDifficulty, Perk, PlayerInGame, PlayerInfo},
};
use chrono;
use diesel::prelude::*;
use log::error;
use std::net::Ipv4Addr;

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::ip_addresses)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub(crate) struct IpAddressDbQ {
    pub(super) id: u32,
    pub(super) steam_id: u64,
    pub(super) ip_address: u32,
    pub(super) created: chrono::NaiveDateTime,
}
#[derive(Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::ip_addresses)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub(crate) struct IpAddressDbI {
    pub(super) steam_id: u64,
    pub(super) ip_address: u32,
}

impl IpAddressDbI {
    pub(crate) fn from(player_steam: PlayerInfo) -> Self {
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
pub(crate) struct PlayerDbQ {
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

#[derive(Insertable, Clone)]
#[diesel(table_name = crate::schema::game_sessions)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub(super) struct GameSessionDbI {
    pub(crate) max_waves: u16,
    pub(crate) reached_wave: u16,
    pub(crate) max_players: u16,
    pub(crate) players_at_most: u16,
    pub(crate) map_name: String,
    pub(crate) difficulty: String,
    pub(crate) game_type: String,
    pub(crate) boss: String,
    pub(crate) started_at: chrono::NaiveDateTime,
    pub(crate) ended_at: Option<chrono::NaiveDateTime>,
}

impl Into<GameSessionDbI> for GameSession {
    fn into(self) -> GameSessionDbI {
        GameSessionDbI {
            max_waves: self.max_waves,
            reached_wave: self.reached_wave,
            max_players: self.max_players,
            players_at_most: self.players_at_most,
            map_name: self.map_name,
            difficulty: self.difficulty.to_string(),
            game_type: self.game_type,
            boss: self.boss,
            started_at: self.started_at,
            ended_at: self.ended_at,
        }
    }
}

#[derive(Clone, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::game_sessions)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub(super) struct GameSessionDbU {
    pub(crate) id: u32,
    pub(crate) max_waves: u16,
    pub(crate) reached_wave: u16,
    pub(crate) max_players: u16,
    pub(crate) players_at_most: u16,
    pub(crate) map_name: String,
    pub(crate) difficulty: String,
    pub(crate) game_type: String,
    pub(crate) boss: String,
    pub(crate) started_at: chrono::NaiveDateTime,
    pub(crate) ended_at: Option<chrono::NaiveDateTime>,
}

impl Into<GameSessionDbU> for GameSession {
    fn into(self) -> GameSessionDbU {
        GameSessionDbU {
            id: self.db_id.expect("no game session id!"),
            max_waves: self.max_waves,
            reached_wave: self.reached_wave,
            max_players: self.max_players,
            players_at_most: self.players_at_most,
            map_name: self.map_name,
            difficulty: self.difficulty.to_string(),
            game_type: self.game_type,
            boss: self.boss,
            started_at: self.started_at,
            ended_at: self.ended_at,
        }
    }
}

#[derive(Clone, Insertable)]
#[diesel(table_name = crate::schema::player_sessions)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub(crate) struct PlayerSessionDbI {
    pub(crate) game_session_id: u32,
    pub(crate) steam_id: u64,
    pub(crate) perk: String,
    pub(crate) kills: u32,
    pub(crate) started_at: chrono::NaiveDateTime,
    pub(crate) ended_at: chrono::NaiveDateTime,
}

impl PlayerSessionDbI {
    pub(crate) fn into_session(self, id: u32) -> PlayerSession {
        PlayerSession {
            db_id: Some(id),
            game_session_id: self.game_session_id,
            steam_id: self.steam_id,
            perk: self.perk,
            kills: self.kills,
            started_at: self.started_at,
            ended_at: self.ended_at,
        }
    }
}

impl Into<PlayerSessionDbI> for PlayerSession {
    fn into(self) -> PlayerSessionDbI {
        PlayerSessionDbI {
            game_session_id: self.game_session_id,
            steam_id: self.steam_id,
            perk: self.perk.to_string(),
            kills: self.kills,
            started_at: self.started_at,
            ended_at: self.ended_at,
        }
    }
}

#[derive(Clone, Insertable, AsChangeset, Selectable)]
#[diesel(table_name = crate::schema::player_sessions)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub(crate) struct PlayerSessionDbU {
    pub(crate) id: u32,
    pub(crate) game_session_id: u32,
    pub(crate) steam_id: u64,
    pub(crate) perk: String,
    pub(crate) kills: u32,
    pub(crate) started_at: chrono::NaiveDateTime,
    pub(crate) ended_at: chrono::NaiveDateTime,
}

impl Into<PlayerSessionDbU> for PlayerSession {
    fn into(self) -> PlayerSessionDbU {
        PlayerSessionDbU {
            id: self.db_id.expect("no player session id!"),
            game_session_id: self.game_session_id,
            steam_id: self.steam_id,
            perk: self.perk.to_string(),
            kills: self.kills,
            started_at: self.started_at,
            ended_at: self.ended_at,
        }
    }
}
