use crate::database::{KfDatabase, PlayerDb};
use crate::parse::{DocumentExtractor, HeaderExtractor};
use reqwest::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{self, Write};
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
    db_connection: KfDatabase,
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
        db_connection: KfDatabase,
    ) -> Result<Self, Box<dyn Error>> {
        let url = Kf2Url::new(ip_addr)?;
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
            db_connection,
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

    pub async fn log_players(&mut self) -> Result<(), Box<dyn Error>> {
        let response1 = self.session.get(self.url.info.as_str()).send();
        let response2 = self.session.get(self.url.players.as_str()).send();

        let (response1_future, response2_future) = tokio::join!(response1, response2);

        let response1 = response1_future?;
        let response2 = response2_future?;

        let text1_future = response1.text();
        let text2_future = response2.text();

        let (text1, text2) = tokio::join!(text1_future, text2_future);

        let document1 = DocumentExtractor::new(&text1?);
        let document2 = DocumentExtractor::new(&text2?);

        let players_in_game = document1.parse_in_game_player_info();
        let players_steam = document2.parse_steam_player_info();

        // println!("{:?}", players_in_game);
        // println!("{:?}", players_steam);

        let players = players_steam.into_iter().map(PlayerDb::from).collect();
        self.db_connection.log_players(players).await?;

        Ok(())
    }
}
