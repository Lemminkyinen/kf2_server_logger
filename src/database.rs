use chrono;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncMysqlConnection, RunQueryDsl};
use std::{error::Error, net::Ipv4Addr};

use crate::model::PlayerInfo;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::players_)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct PlayerDb {
    steam_id: u64,
    name: String,
    count: u32,
    ip_address: u32,
    ping: u32,
    unique_net_id: String,
    last_joined: chrono::NaiveDateTime,
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
pub struct KfDatabase {
    ip_addr: String,
    username: String,
    password: String,
    connection: AsyncMysqlConnection,
}

impl KfDatabase {
    pub async fn new_session(
        ip_addr: String,
        db_name: String,
        username: String,
        password: String,
    ) -> Result<Self, Box<dyn Error>> {
        let database_url = format!("mysql://{}:{}@{}/{}", username, password, ip_addr, db_name);
        println!("Connecting to database: {}", database_url);
        let connection = AsyncMysqlConnection::establish(&database_url).await?;
        Ok(Self {
            ip_addr,
            username,
            password,
            connection,
        })
    }

    pub async fn log_players(&mut self, players: Vec<PlayerDb>) -> Result<(), Box<dyn Error>> {
        use crate::schema::players_;

        let asd = diesel::insert_into(players_::table)
            .values(players)
            .execute(&mut self.connection)
            .await?;
        Ok(())
    }
}
