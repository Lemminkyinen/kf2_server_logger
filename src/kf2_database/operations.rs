use super::models::KfDbManager;
use super::models_db::{GameSessionDbI, PlayerDbI};
use crate::kf2_database::models_db::GameSessionDbU;
use crate::kf2_log::logger::GameSession;
use crate::{
    kf2_database::models_db::{CurrentPlayer, IpAddressDbI, IpAddressDbQ, PlayerDbQ},
    kf2_scrape::models::{GameInfo, PlayerInGame, PlayerInfo},
};
use diesel::mysql::MysqlConnection;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use log::{debug, error, info, log_enabled, warn, Level};
use std::{error::Error, net::Ipv4Addr, thread::current};
use std::{io::ErrorKind, thread};

impl KfDbManager {
    pub(crate) async fn log_unique_players(
        &mut self,
        players: Vec<PlayerInfo>,
    ) -> Result<(), Box<dyn Error>> {
        if players.is_empty() {
            return Ok(());
        }
        let mut connection1 = self.get_connection()?;
        let mut connection2 = self.get_connection()?;
        let players2 = players.clone();

        let thread_players = thread::spawn(move || {
            if let Err(err) = Self::insert_unique_players(&mut connection1, players) {
                return Err(std::io::Error::new(ErrorKind::Other, err.to_string()));
            };
            Ok(())
        });
        thread_players.join().unwrap()?;
        let thread_ip = thread::spawn(move || {
            if let Err(err) = Self::insert_ip_addresses(&mut connection2, players2) {
                return Err(std::io::Error::new(ErrorKind::Other, err.to_string()));
            };
            Ok(())
        });
        thread_ip.join().unwrap()?;
        Ok(())
    }

    pub(super) fn insert_ip_addresses(
        connection: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
        players: Vec<PlayerInfo>,
    ) -> Result<(), Box<dyn Error>> {
        use crate::schema::ip_addresses::dsl::*;

        let existing_ip_addresses: Vec<IpAddressDbQ> = ip_addresses
            .filter(steam_id.eq_any(players.iter().map(|p| &p.steam_id)))
            .load::<IpAddressDbQ>(connection)?;

        let new_ip_addresses = players
            .into_iter()
            .filter(|p| {
                existing_ip_addresses.iter().all(|ip| {
                    let player_ip: u32 = match p.ip {
                        std::net::IpAddr::V4(ip) => ip.into(),
                        ip => {
                            error!("Invalid IP address {}", ip);
                            Ipv4Addr::new(0, 0, 0, 0).into()
                        }
                    };
                    !(ip.steam_id == p.steam_id && ip.ip_address == player_ip)
                })
            })
            .map(|p| IpAddressDbI::from(p))
            .collect::<Vec<_>>();

        if new_ip_addresses.len() > 0 {
            let len = new_ip_addresses.len();
            diesel::insert_into(ip_addresses)
                .values(new_ip_addresses)
                .execute(connection)?;
            info!("Added {} new ip addresses", len);
        }
        Ok(())
    }

    pub(super) fn insert_unique_players(
        connection: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
        players: Vec<PlayerInfo>,
    ) -> Result<(), Box<dyn Error>> {
        use crate::schema::unique_players::dsl::*;
        let players = players
            .into_iter()
            .map(|p| PlayerDbI::from(p))
            .collect::<Vec<_>>();

        let mut existing_players_db = unique_players
            .filter(steam_id.eq_any(players.iter().map(|p| &p.steam_id)))
            .load::<PlayerDbQ>(connection)?;

        let mut existing_players = players
            .clone()
            .into_iter()
            .filter(|p| {
                existing_players_db
                    .iter()
                    .any(|ep| ep.steam_id == p.steam_id)
            })
            .collect::<Vec<_>>();

        let new_players = players
            .into_iter()
            .filter(|p| {
                !existing_players_db
                    .iter()
                    .any(|ep| ep.steam_id == p.steam_id)
            })
            .collect::<Vec<_>>();

        if !new_players.is_empty() {
            let len = new_players.len();
            diesel::insert_into(unique_players)
                .values(new_players)
                .execute(connection)?;
            info!("Added {} new unique players", len);
        }

        if existing_players.len() == existing_players_db.len() {
            existing_players.sort_by(|a, b| a.steam_id.cmp(&b.steam_id));
            existing_players_db.sort_by(|a, b| a.steam_id.cmp(&b.steam_id));

            let zipped = existing_players
                .into_iter()
                .zip(existing_players_db.into_iter());

            for (player, player_db) in zipped {
                if player.steam_id != player_db.steam_id {
                    error!(
                        "Player steam id mismatch: {} != {}",
                        player.steam_id, player_db.steam_id
                    );
                }
                let p_name = player.name;
                // let p_count = player_db.maps_played;
                let p_ping = if player_db.avg_ping == 0 {
                    player.avg_ping
                } else {
                    (player_db.avg_ping + player.avg_ping) / 2
                };
                diesel::update(unique_players.find(player.steam_id))
                    .set((
                        name.eq(p_name.clone()),
                        // maps_played.eq(p_count),
                        avg_ping.eq(p_ping),
                        unique_net_id.eq(player.unique_net_id),
                    ))
                    .execute(connection)?;
                info!("Updated player: {}", p_name);
            }
        }
        Ok(())
    }

    pub(super) fn clean_current_players(
        connection: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
    ) -> Result<(), Box<dyn Error>> {
        use crate::schema::current_players::dsl::*;
        diesel::delete(current_players).execute(connection)?;
        Ok(())
    }

    pub(super) fn insert_current_players(
        connection: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
        players: Vec<PlayerInGame>,
    ) -> Result<(), Box<dyn Error>> {
        use crate::schema::current_players::dsl::*;
        let players = players
            .into_iter()
            .map(|p| p.into())
            .collect::<Vec<CurrentPlayer>>();
        diesel::insert_into(current_players)
            .values(players)
            .execute(connection)?;
        Ok(())
    }

    pub(crate) async fn log_in_game_players(
        &mut self,
        players: Vec<PlayerInGame>,
    ) -> Result<(), Box<dyn Error>> {
        let mut connection = self.get_connection()?;
        Self::clean_current_players(&mut connection)?;
        if players.is_empty() {
            return Ok(());
        }
        Self::insert_current_players(&mut connection, players)?;
        Ok(())
    }

    pub(super) fn insert_game_session(
        connection: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
        game_session: GameSessionDbI,
    ) -> Result<u32, Box<dyn Error>> {
        use crate::schema::game_sessions::dsl::*;
        let result = diesel::insert_into(game_sessions)
            .values(game_session)
            .execute(connection)?;
        let db_id = game_sessions
            .select(diesel::dsl::max(id))
            .load::<Option<u32>>(connection)?
            .first()
            .ok_or("no game session rows")?
            .ok_or("no game session id ")?;
        Ok(db_id)
    }

    pub(super) fn update_game_session(
        connection: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
        game_session: GameSessionDbU,
    ) -> Result<(), Box<dyn Error>> {
        use crate::schema::game_sessions::dsl::*;
        diesel::update(game_sessions.find(game_session.id))
            .set(game_session)
            .execute(connection)?;
        Ok(())
    }

    pub(crate) async fn log_game_session(
        &mut self,
        game_info: GameSession,
    ) -> Result<u32, Box<dyn Error>> {
        let mut connection = self.get_connection()?;
        let db_id;
        if let Some(id) = game_info.db_id {
            db_id = id;
            let game_session = game_info.into();
            Self::update_game_session(&mut connection, game_session)?;
        } else {
            let game_session = game_info.into();
            db_id = Self::insert_game_session(&mut connection, game_session)?;
        }
        Ok(db_id)
    }
}
