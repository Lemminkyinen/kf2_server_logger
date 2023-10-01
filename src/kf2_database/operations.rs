use super::models::KfDbManager;
use super::models_db::PlayerDbI;
use crate::{
    kf2_database::models_db::{IpAddressDbI, IpAddressDbQ, PlayerDbQ},
    kf2_scrape::models::PlayerInfo,
};
use diesel::dsl::*;
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::{BelongingToDsl, ExpressionMethods, QueryDsl, RunQueryDsl};
use log::{debug, error, info, log_enabled, warn, Level};
use std::{error::Error, net::Ipv4Addr};
use std::{io::ErrorKind, thread};

impl KfDbManager {
    pub async fn log_unique_players(
        &mut self,
        players: Vec<PlayerInfo>,
    ) -> Result<(), Box<dyn Error>> {
        let mut connection1 = self.get_connection()?;
        let mut connection2 = self.get_connection()?;
        let players2 = players.clone();

        let thread_players = thread::spawn(move || {
            if let Err(err) = Self::insert_unique_players(&mut connection1, players) {
                return Err(std::io::Error::new(ErrorKind::Other, err.to_string()));
            };
            Ok(())
        });
        let thread_ip = thread::spawn(move || {
            if let Err(err) = Self::insert_ip_addresses(&mut connection2, players2) {
                return Err(std::io::Error::new(ErrorKind::Other, err.to_string()));
            };
            Ok(())
        });

        thread_players.join().unwrap()?;
        thread_ip.join().unwrap()?;
        Ok(())
    }

    fn insert_ip_addresses(
        connection: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
        players: Vec<PlayerInfo>,
    ) -> Result<(), Box<dyn Error>> {
        use crate::schema::ip_addresses::dsl::*;

        // let asd = ip_addresses.filter(steam_id.eq_any(players.iter().map(|p| &p.steam_id)));

        let existing_ip_addresses = ip_addresses
            .filter(steam_id.eq_any(players.iter().map(|p| &p.steam_id)))
            .load::<IpAddressDbQ>(connection)?;

        let new_ip_addresses = players
            .into_iter()
            .filter(|p| {
                existing_ip_addresses.iter().any(|ip| {
                    let player_ip: u32 = match p.ip {
                        std::net::IpAddr::V4(ip) => ip.into(),
                        ip => {
                            error!("Invalid IP address {}", ip);
                            Ipv4Addr::new(0, 0, 0, 0).into()
                        }
                    };
                    ip.steam_id != p.steam_id && ip.ip_address != player_ip
                })
            })
            .map(|p| IpAddressDbI::from(p))
            .collect::<Vec<IpAddressDbI>>();

        if new_ip_addresses.len() > 0 {
            let len = new_ip_addresses.len();
            diesel::insert_into(ip_addresses)
                .values(new_ip_addresses)
                .execute(connection)?;
            info!("Added {} new ip addresses", len);
        }
        Ok(())
    }

    fn insert_unique_players(
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
                        // ip_address.eq(player.ip_address),
                        unique_net_id.eq(player.unique_net_id),
                    ))
                    .execute(connection)?;
                info!("Updated player: {}", p_name);
            }
        }
        Ok(())
    }
}
