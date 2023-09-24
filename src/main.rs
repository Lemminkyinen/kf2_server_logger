mod database;
mod kf2;
mod model;
mod parse;

use clap::Parser;
use kf2::Kf2Logger;
use log::{debug, error, info, log_enabled, warn, Level};
use url::Url;

#[derive(Debug, Parser)]
struct Args {
    server_ip: Url,
    username: String,
    password: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = Args::parse();
    let kf2 = Kf2Logger::new_session(args.server_ip, args.username, args.password)
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
