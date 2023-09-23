use crate::parse::{DocumentExtractor, HeaderExtractor};
use reqwest::{dns::Resolving, Client, ClientBuilder};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Write};
use std::{error::Error, num::ParseIntError};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthForm {
    pub token: String,
    pub password_hash: String,
    pub username: String,
    pub password: String,
    pub remember: String,
}

pub struct Kf2Url {
    pub base: Url,
    pub web_admin: Url,
    pub info: Url,
    pub players: Url,
}

impl Kf2Url {
    pub fn new(base_url: Url) -> Result<Self, Box<dyn Error>> {
        let base = base_url.clone();
        let web_admin = base_url.join("ServerAdmin/")?;
        let info = web_admin.join("current/info")?;
        let players = web_admin.join("current/players")?;

        Ok(Self {
            base,
            web_admin,
            info,
            players,
        })
    }
}

pub struct Kf2Logger {
    url: Kf2Url,
    session: Client,
    session_id: String,
    auth_cred: Option<String>,
}

fn write_text_to_file(filename: &str, text: &str) -> io::Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(text.as_bytes())?;
    Ok(())
}

impl Kf2Logger {
    pub async fn new_session(
        ip_addr: Url,
        username: String,
        password: String,
    ) -> Result<Self, Box<dyn Error>> {
        let base_url = ip_addr.clone();
        drop(ip_addr);
        let url = Kf2Url::new(base_url)?;
        let client = ClientBuilder::new().cookie_store(true).build()?;
        let get_response = client.get(url.web_admin.as_str()).send().await?;
        let headers = HeaderExtractor::new(get_response.headers().to_owned());
        let text = get_response.text().await?;
        let document = DocumentExtractor::new(&text);
        drop(text);
        let token = document.parse_form_token()?;
        let session_id = headers.get_cookie("sessionid").unwrap();

        let form = AuthForm {
            token,
            password_hash: "".to_string(),
            username,
            password,
            remember: "-1".to_string(),
        };

        let post_response = client
            .post(url.web_admin.as_str())
            .form(&form)
            .send()
            .await?;
        let headers = HeaderExtractor::new(post_response.headers().to_owned());
        let auth_cred = headers.get_cookie("authcred");
        let session = client;

        Ok(Self {
            url,
            session,
            session_id,
            auth_cred,
        })
    }

    async fn get_info(&self) -> Result<(), Box<dyn Error>> {
        // let response = self.session.get(self.info_url.clone()).send().await?;
        // let text = response.text().await?;
        // println!("{:?}", &headers);
        // println!("{:?}", &text);
        // let document = DocumentExtractor::new(&text);
        // let info = document.parse_info()?;
        todo!()
    }

    pub async fn log_all(&self) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    pub async fn log_players(&self) -> Result<(), Box<dyn Error>> {
        let response = self.session.get(self.url.info.as_str()).send().await?;
        let text = response.text().await?;
        let document = DocumentExtractor::new(&text);
        let players_in_game = document.parse_in_game_player_info();

        let response = self.session.get(self.url.players.as_str()).send().await?;
        let text = response.text().await?;
        let document = DocumentExtractor::new(&text);
        let players_steam = document.parse_steam_player_info();

        println!("{:?}", players_in_game);
        println!("{:?}", players_steam);

        Ok(())
    }
}
