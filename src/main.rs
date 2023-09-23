mod database;
mod kf2;
mod model;
mod parse;

use clap::Parser;
use kf2::Kf2Logger;
use url::Url;

#[derive(Debug, Parser)]
struct Args {
    server_ip: Url,
    username: String,
    password: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let kf2 = Kf2Logger::new_session(args.server_ip, args.username, args.password)
        .await
        .unwrap();
    kf2.log_players().await.unwrap();
}
