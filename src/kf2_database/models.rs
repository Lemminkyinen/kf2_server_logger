use crate::kf2_scrape::models::PlayerInfo;
use chrono;
use diesel::prelude::*;
use diesel_async::AsyncMysqlConnection;
use std::net::Ipv4Addr;

pub struct KfDbManager {
    pub(super) ip_addr: String,
    pub(super) username: String,
    pub(super) password: String,
    pub(super) connection: AsyncMysqlConnection,
}
