use crate::args::Kf2ServerArgs;
use crate::kf2_database::models::KfDbManager;
use crate::kf2_scrape::models::{GameInfo, KfDifficulty, PlayerInGame};
use crate::kf2_scrape::parse::{DocumentExtractor, HeaderExtractor};
use reqwest::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{self, Write};
use url::Url;

#[derive(Debug, Clone)]
pub(crate) struct GameSession {
    pub(crate) db_id: Option<u32>,
    pub(crate) max_waves: u16,
    pub(crate) reached_wave: u16,
    pub(crate) max_players: u16,
    pub(crate) players_at_most: u16,
    pub(crate) map_name: String,
    pub(crate) difficulty: KfDifficulty,
    pub(crate) game_type: String,
    pub(crate) boss: String,
    pub(crate) started_at: chrono::NaiveDateTime,
    pub(crate) ended_at: Option<chrono::NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct AuthForm {
    pub token: String,
    pub password_hash: String,
    pub username: String,
    pub password: String,
    pub remember: String,
}

pub(super) struct Kf2Url {
    pub base: Url,
    pub web_admin: Url,
    pub info: Url,
    pub players: Url,
}

pub(crate) struct Kf2Logger {
    url: Kf2Url,
    session: Client,
    db_connection: KfDbManager,
    session_id: String,
    auth_cred: Option<String>,
    game_session: Option<GameSession>,
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

fn write_text_to_file(filename: &str, text: &str) -> io::Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(text.as_bytes())?;
    Ok(())
}

impl Kf2Logger {
    pub(crate) async fn new_session(
        args: Kf2ServerArgs,
        db_connection: KfDbManager,
    ) -> Result<Self, Box<dyn Error>> {
        let (ip_addr, username, password) = args.get();
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
            game_session: None,
        })
    }

    pub async fn log_all(&self) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    pub(crate) async fn log_unique_players(&mut self) -> Result<(), Box<dyn Error>> {
        let response = self.session.get(self.url.players.as_str()).send().await?;
        let text = response.text().await?;
        let document = DocumentExtractor::new(&text);
        let players_steam = document.parse_steam_player_info();
        if players_steam.is_empty() {
            return Ok(());
        }
        self.db_connection.log_unique_players(players_steam).await?;
        Ok(())
    }

    pub(crate) async fn loq_in_game_players(&mut self) -> Result<(), Box<dyn Error>> {
        let response = self.session.get(self.url.info.as_str()).send().await?;
        let text = response.text().await?;
        let document = DocumentExtractor::new(&text);
        let players_in_game = document.parse_in_game_player_info();
        if players_in_game.is_empty() {
            return Ok(());
        }
        self.db_connection
            .log_in_game_players(players_in_game)
            .await?;
        Ok(())
    }

    pub(crate) async fn log_game_session(&mut self) -> Result<(), Box<dyn Error>> {
        let response = self.session.get(self.url.info.as_str()).send().await?;
        let text = response.text().await?;
        let document = DocumentExtractor::new(&text);
        let game_info = document.parse_current_map_info()?;

        if game_info.current_players == 0 && game_info.current_wave == 0 {
            self.game_session = None;
            return Ok(());
        }
        if let Some(game_session) = &mut self.game_session {
            let reached_wave = if game_info.current_wave > game_session.reached_wave {
                game_info.current_wave
            } else {
                game_session.reached_wave
            };
            let players_at_most = if game_info.current_players > game_session.players_at_most {
                game_info.current_players
            } else {
                game_session.players_at_most
            };
            game_session.reached_wave = reached_wave;
            game_session.players_at_most = players_at_most;
            game_session.ended_at = Some(chrono::Local::now().naive_utc());
            self.db_connection
                .log_game_session(game_session.clone())
                .await?;
        } else {
            let boss = "get_boss()".to_string();
            let mut game_session = GameSession {
                db_id: None,
                max_waves: game_info.max_waves,
                reached_wave: game_info.current_wave,
                max_players: game_info.max_players,
                players_at_most: game_info.current_players,
                map_name: game_info.map_name,
                difficulty: game_info.difficulty,
                game_type: game_info.game_type,
                boss: boss,
                started_at: chrono::Local::now().naive_utc(),
                ended_at: None,
            };
            let db_id = self
                .db_connection
                .log_game_session(game_session.clone())
                .await?;
            game_session.db_id = Some(db_id);
            self.game_session = Some(game_session.clone());
        }
        Ok(())
    }
}
