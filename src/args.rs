use dotenv::dotenv;
use std::env;
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

#[derive(Debug)]
struct Args {
    web_admin_url: Url,
    web_admin_username: String,
    web_admin_password: String,
    database_url: String,
    database_name: String,
    database_username: String,
    database_password: String,
}

/// Read arguments from .env file
pub fn parse() -> (Kf2ServerArgs, Kf2DbArgs) {
    dotenv().ok();

    fn get_env(s: &str) -> String {
        fn fmt_env_msg(s: &str) -> String {
            format!("Could not read {s} environment variable!")
        }

        env::var(s).unwrap_or_else(|_| panic!("{}", fmt_env_msg(s)))
    }

    let web_admin_url = "WEB_ADMIN_URL";
    let web_admin_username = "WEB_ADMIN_USERNAME";
    let web_admin_password = "WEB_ADMIN_PASSWORD";
    let database_url = "DATABASE_URL";
    let database_name = "DATABASE_NAME";
    let database_username = "DATABASE_USERNAME";
    let database_password = "DATABASE_PASSWORD";

    let db_args = Kf2DbArgs {
        server_address: get_env(database_url),
        database: get_env(database_name),
        username: get_env(database_username),
        password: get_env(database_password),
    };

    let url_str = get_env(web_admin_url);
    let web_admin_url =
        Url::parse(&url_str).unwrap_or_else(|_| panic!("Could not parse url: {url_str}"));

    let server_args = Kf2ServerArgs {
        server_ip: web_admin_url,
        username: get_env(web_admin_username),
        password: get_env(web_admin_password),
    };
    (server_args, db_args)
}
