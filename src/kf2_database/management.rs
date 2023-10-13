use crate::args::Kf2DbArgs;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::r2d2::PooledConnection;
use diesel::MysqlConnection;
use std::error::Error;

pub struct KfDbManager {
    pub(super) ip_addr: String,
    pub(super) username: String,
    pub(super) password: String,
    pub(super) pool: Pool<ConnectionManager<MysqlConnection>>,
}

impl KfDbManager {
    pub(crate) fn new_session(args: Kf2DbArgs) -> Result<Self, Box<dyn Error>> {
        let database_url = args.get_connection_string();
        let (username, password, ip_addr, _) = args.get();
        let manager = ConnectionManager::<MysqlConnection>::new(database_url);
        let pool = Pool::builder().test_on_check_out(true).build(manager)?;
        Ok(Self {
            ip_addr,
            username,
            password,
            pool,
        })
    }

    pub(super) fn get_connection(
        &self,
    ) -> Result<PooledConnection<ConnectionManager<MysqlConnection>>, Box<dyn Error>> {
        let new_pool = self.pool.clone();
        Ok(new_pool.get()?)
    }
}
