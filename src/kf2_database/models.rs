use diesel::{
    r2d2::{ConnectionManager, Pool},
    MysqlConnection,
};

pub struct KfDbManager {
    pub(super) ip_addr: String,
    pub(super) username: String,
    pub(super) password: String,
    pub(super) pool: Pool<ConnectionManager<MysqlConnection>>,
}
