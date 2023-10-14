mod args;
mod kf2_database;
mod kf2_log;
mod kf2_scrape;
pub mod schema;

use kf2_database::management::KfDbManager;
use kf2_log::logger::Kf2Logger;
use log::{error, info};
use tokio::time::Instant;

#[tokio::main]
async fn main() {
    env_logger::init();
    let (server_args, db_args) = args::parse();

    let kf2db = KfDbManager::new_session(db_args).unwrap();
    let mut kf2 = Kf2Logger::new_session(server_args, kf2db).await.unwrap();

    let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
    '_log: loop {
        interval.tick().await;
        let start = Instant::now();
        if let Err(err) = kf2.log_unique_players().await {
            error!("{}", err);
        }
        let duration = start.elapsed();
        info!("Log Unique Players Duration: {:?}", duration);
        let start = Instant::now();
        if let Err(err) = kf2.loq_in_game_players().await {
            error!("{}", err);
        }
        let duration = start.elapsed();
        info!("Loq In Game Players Duration: {:?}", duration);
        let start = Instant::now();
        if let Err(err) = kf2.log_game_session().await {
            error!("{}", err);
        }
        let duration = start.elapsed();
        info!("Log Game Session Duration: {:?}", duration);
        let start = Instant::now();
        if let Err(err) = kf2.log_player_sessions().await {
            error!("{}", err);
        }
        let duration = start.elapsed();
        info!("Log Player Sessions Duration: {:?}", duration);
    }
}
