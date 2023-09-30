use super::models::KfDbManager;
use crate::args::Kf2DbArgs;
use diesel_async::{AsyncConnection, AsyncMysqlConnection};
use std::error::Error;

impl KfDbManager {
    pub async fn new_session(args: Kf2DbArgs) -> Result<Self, Box<dyn Error>> {
        let database_url = args.get_connection_string();
        let (username, password, ip_addr, _) = args.get();
        let connection = AsyncMysqlConnection::establish(&database_url).await?;
        Ok(Self {
            ip_addr,
            username,
            password,
            connection,
        })
    }

    pub fn get_connection(&mut self) -> &mut AsyncMysqlConnection {
        &mut self.connection
    }
}
