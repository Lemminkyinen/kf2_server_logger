mod database;
mod kf2;
mod model;
mod parse;
pub mod schema;

use clap::Parser;
use database::KfDatabase;
use kf2::Kf2Logger;
use log::{debug, error, info, log_enabled, warn, Level};
use url::Url;

#[derive(Debug, Parser)]
struct Args {
    kf2_server_ip: Url,
    kf2_username: String,
    kf2_password: String,
    db_server: String,
    db_database: String,
    db_username: String,
    db_password: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();

    let kf2db = KfDatabase::new_session(
        args.db_server,
        args.db_database,
        args.db_username,
        args.db_password,
    )
    .await
    .unwrap();

    let mut kf2 = Kf2Logger::new_session(
        args.kf2_server_ip,
        args.kf2_username,
        args.kf2_password,
        kf2db,
    )
    .await
    .unwrap();

    let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
    loop {
        interval.tick().await;
        if let Err(err) = kf2.log_players().await {
            error!("{}", err);
        } else {
            info!("Players logged");
        }
    }
}
