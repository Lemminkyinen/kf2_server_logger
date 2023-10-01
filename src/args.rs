use std::error::Error;

use clap::Parser;
use url::Url;

#[derive(Debug)]
pub struct Kf2ServerArgs {
    server_ip: Url,
    username: String,
    password: String,
}

impl Kf2ServerArgs {
    pub fn get(self) -> (Url, String, String) {
        (self.server_ip, self.username, self.password)
    }
}

#[derive(Debug)]
pub struct Kf2DbArgs {
    pub(super) server_address: String,
    pub(super) database: String,
    pub(super) username: String,
    pub(super) password: String,
}

impl Kf2DbArgs {
    pub fn get(self) -> (String, String, String, String) {
        (
            self.server_address,
            self.database,
            self.username,
            self.password,
        )
    }
    pub fn get_connection_string(&self) -> String {
        format!(
            "mysql://{}:{}@{}/{}",
            self.username, self.password, self.server_address, self.database
        )
    }
}

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

pub fn parse() -> (Kf2ServerArgs, Kf2DbArgs) {
    let args = Args::parse();
    let db_args = Kf2DbArgs {
        server_address: args.db_server,
        database: args.db_database,
        username: args.db_username,
        password: args.db_password,
    };
    let server_args = Kf2ServerArgs {
        server_ip: args.kf2_server_ip,
        username: args.kf2_username,
        password: args.kf2_password,
    };
    (server_args, db_args)
}
