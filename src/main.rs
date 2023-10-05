mod args;
mod kf2_database;
mod kf2_log;
mod kf2_scrape;
pub mod schema;

use kf2_database::models::KfDbManager;
use kf2_log::logger::Kf2Logger;
use log::{debug, error, info, log_enabled, warn, Level};

#[tokio::main]
async fn main() {
    env_logger::init();
    let (server_args, db_args) = args::parse();

    let kf2db = KfDbManager::new_session(db_args).unwrap();
    let mut kf2 = Kf2Logger::new_session(server_args, kf2db).await.unwrap();

    let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
    'log: loop {
        interval.tick().await;
        if let Err(err) = kf2.log_unique_players().await {
            error!("{}", err);
        }
        if let Err(err) = kf2.loq_in_game_players().await {
            error!("{}", err);
        }
        if let Err(err) = kf2.log_game_session().await {
            error!("{}", err);
        }
    }
}
