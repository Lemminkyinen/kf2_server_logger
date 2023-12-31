use crate::args::Kf2ServerArgs;
use crate::kf2_database::management::KfDbManager;
use crate::kf2_database::models::PlayerSessionDbU;
use crate::kf2_scrape::models::{GameInfo, KfDifficulty, PlayerInGame, PlayerInfo};
use crate::kf2_scrape::parse::{DocumentExtractor, HeaderExtractor};
use log::info;
use reqwest::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use url::Url;

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Boss {
    HansVolter = 0,
    Patriarch = 1,
    KingFleshpound = 2,
    Matriarch = 3,
    Abomination = 4,
    Undefined = 5,
}

impl Boss {
    pub(crate) fn map(input: &u8) -> Result<Self, Box<dyn Error>> {
        match input {
            0 => Ok(Boss::HansVolter),
            1 => Ok(Boss::Patriarch),
            2 => Ok(Boss::KingFleshpound),
            3 => Ok(Boss::Matriarch),
            4 => Ok(Boss::Abomination),
            _ => Err(format!("Unknown boss {}", input).into()),
        }
    }

    pub(crate) fn value(&self) -> u8 {
        self.clone() as u8
    }

    pub(crate) fn to_string(&self) -> String {
        match self {
            Boss::HansVolter => String::from("Hans Volter"),
            Boss::Patriarch => String::from("Patriarch"),
            Boss::KingFleshpound => String::from("King Fleshpound"),
            Boss::Matriarch => String::from("Matriarch"),
            Boss::Abomination => String::from("Abomination"),
            Boss::Undefined => String::from("Undefined"),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PlayerSession {
    pub(crate) db_id: Option<u32>,
    pub(crate) game_session_id: u32,
    pub(crate) steam_id: u64,
    pub(crate) perk: String,
    pub(crate) kills: u32,
    pub(crate) started_at: chrono::NaiveDateTime,
    pub(crate) ended_at: chrono::NaiveDateTime,
}

impl Into<PlayerSession> for PlayerSessionDbU {
    fn into(self) -> PlayerSession {
        PlayerSession {
            db_id: Some(self.id),
            game_session_id: self.game_session_id,
            steam_id: self.steam_id,
            perk: self.perk,
            kills: self.kills,
            started_at: self.started_at,
            ended_at: self.ended_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SessionStatus {
    New,
    InProgress,
    Ended,
}

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
    pub(crate) boss: Boss,
    pub(crate) started_at: chrono::NaiveDateTime,
    pub(crate) ended_at: Option<chrono::NaiveDateTime>,
    pub(crate) status: SessionStatus,
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
    pub(super) base: Url,
    pub(super) web_admin: Url,
    pub(super) info: Url,
    pub(super) players: Url,
    pub(super) console: Url,
}

pub(crate) struct Kf2Logger {
    url: Kf2Url,
    session: Client,
    db_connection: KfDbManager,
    session_id: String,
    auth_cred: Option<String>,
    game_session: Option<GameSession>,
    in_game_players: Option<Vec<PlayerInGame>>,
    unique_players: Option<Vec<PlayerInfo>>,
    player_sessions: Option<Vec<PlayerSession>>,
}

impl Kf2Url {
    pub fn new(base_url: Url) -> Result<Self, Box<dyn Error>> {
        let base = base_url.clone();
        let web_admin = base_url.join("ServerAdmin/")?;
        let info = web_admin.join("current/info")?;
        let players = web_admin.join("current/players")?;
        let console = web_admin.join("console")?;

        Ok(Self {
            base,
            web_admin,
            info,
            players,
            console,
        })
    }
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
        let session_id = headers
            .get_cookie("sessionid")
            .ok_or("Session id not found")?;

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
            in_game_players: None,
            unique_players: None,
            player_sessions: None,
        })
    }

    async fn get_unique_players(&self) -> Result<Vec<PlayerInfo>, Box<dyn Error>> {
        let response = self.session.get(self.url.players.as_str()).send().await?;
        let text = response.text().await?;
        let document = DocumentExtractor::new(&text);
        Ok(document.parse_steam_player_info())
    }

    pub(crate) async fn log_unique_players(&mut self) -> Result<(), Box<dyn Error>> {
        let players_steam = self.get_unique_players().await?;
        if players_steam.is_empty() {
            self.unique_players = None;
            return Ok(());
        }
        self.unique_players = Some(players_steam.clone());
        self.db_connection.log_unique_players(players_steam).await?;
        Ok(())
    }

    async fn get_in_game_players(&self) -> Result<Vec<PlayerInGame>, Box<dyn Error>> {
        let response = self.session.get(self.url.info.as_str()).send().await?;
        let text = response.text().await?;
        let document = DocumentExtractor::new(&text);
        Ok(document.parse_in_game_player_info())
    }

    pub(crate) async fn loq_in_game_players(&mut self) -> Result<(), Box<dyn Error>> {
        let players_in_game = self.get_in_game_players().await?;
        self.in_game_players = if players_in_game.is_empty() {
            None
        } else {
            Some(players_in_game.clone())
        };
        self.db_connection
            .log_in_game_players(players_in_game)
            .await?;
        Ok(())
    }

    async fn get_boss_info(&self) -> Result<Boss, Box<dyn Error>> {
        let form_data = HashMap::from([(
            String::from("command"),
            String::from("getall KFGameReplicationInfo BossIndex"),
        )]);
        let response = self
            .session
            .post(self.url.console.as_str())
            .form(&form_data)
            .send()
            .await?;
        let text = response.text().await?;
        let document = DocumentExtractor::new(&text);
        let boss_index = document.parse_current_boss_info()?;
        Ok(Boss::map(&boss_index)?)
    }

    async fn get_game_session(&self) -> Result<GameInfo, Box<dyn Error>> {
        let response = self.session.get(self.url.info.as_str()).send().await?;
        let text = response.text().await?;
        let document = DocumentExtractor::new(&text);
        Ok(document.parse_current_map_info()?)
    }

    pub(crate) async fn log_game_session(&mut self) -> Result<(), Box<dyn Error>> {
        let mut game_info = self.get_game_session().await?;
        game_info.boss = self.get_boss_info().await?;
        if game_info.current_players == 0 {
            self.game_session = None;
            return Ok(());
        }
        if let Some(game_session) = &mut self.game_session {
            if game_session.map_name != game_info.map_name
                || game_session.boss != game_info.boss
                || game_session.reached_wave > game_info.current_wave
            {
                self.game_session = None;
            }
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
            game_session.ended_at = Some(chrono::Utc::now().naive_utc());

            if let Some(players) = &self.in_game_players {
                if players.iter().any(|p| p.kills > 0) {
                    game_session.db_id = Some(
                        self.db_connection
                            .log_game_session(game_session.clone())
                            .await?,
                    );
                    game_session.status = SessionStatus::InProgress;
                } else {
                    info!("Game session not saved to database. Game session exist, but no in game players that have kills.");
                }
            } else {
                info!("Game session not saved to database. Game session exist, but no in game players.");
            }
        } else {
            let game_session = GameSession {
                db_id: None,
                max_waves: game_info.max_waves,
                reached_wave: game_info.current_wave,
                max_players: game_info.max_players,
                players_at_most: game_info.current_players,
                map_name: game_info.map_name,
                difficulty: game_info.difficulty,
                game_type: game_info.game_type,
                boss: game_info.boss,
                started_at: chrono::Utc::now().naive_utc(),
                ended_at: None,
                status: SessionStatus::New,
            };
            self.game_session = Some(game_session);
        }
        Ok(())
    }

    async fn create_new_players_sessions(&mut self) -> Result<Vec<PlayerSession>, Box<dyn Error>> {
        if self.game_session.is_none()
            && self.unique_players.is_none()
            && self.in_game_players.is_none()
        {
            return Ok(vec![]);
        }
        if let Some(game_session) = &self.game_session {
            if game_session.db_id.is_none() {
                return Ok(vec![]);
            }
        }
        let player_sessions;
        let game_session = self
            .game_session
            .as_ref()
            .ok_or("Game session not found, when trying to build players sessions")?;
        let unique_players = self
            .unique_players
            .as_mut()
            .ok_or("Unique players not found, when trying to build players sessions")?;
        let in_game_players = self
            .in_game_players
            .as_mut()
            .ok_or("In game players not found, when trying to build players sessions")?;

        unique_players.sort_by(|a, b| a.name.cmp(&b.name));
        in_game_players.sort_by(|a, b| a.name.cmp(&b.name));

        if unique_players.len() == in_game_players.len() {
            player_sessions = unique_players
                .into_iter()
                .zip(in_game_players)
                .map(|(unique_player, in_game_player)| PlayerSession {
                    db_id: None,
                    game_session_id: game_session.db_id.expect("Game session id not found!"),
                    steam_id: unique_player.steam_id,
                    perk: in_game_player.perk.to_string(),
                    kills: in_game_player.kills,
                    started_at: chrono::Utc::now().naive_utc(),
                    ended_at: chrono::Utc::now().naive_utc(),
                })
                .collect()
        } else {
            return Err("Unique players and in game players are not the same size".into());
        }
        Ok(player_sessions)
    }

    async fn update_player_sessions(
        &mut self,
        new_player_sessions: Vec<PlayerSession>,
    ) -> Result<Vec<PlayerSession>, Box<dyn Error>> {
        let mut updated_player_sessions = vec![];
        if let Some(old_player_sessions) = self.player_sessions.as_mut() {
            // Add new player sessions to the sessions list
            updated_player_sessions.extend(new_player_sessions.clone().into_iter().filter(
                |new_player_session| {
                    old_player_sessions
                        .iter()
                        .find(|old_player_session| {
                            old_player_session.steam_id == new_player_session.steam_id
                        })
                        .is_none()
                        && new_player_session.kills > 0
                },
            ));

            // Add updated existing players to the sessions list
            updated_player_sessions.extend(old_player_sessions.clone().into_iter().filter_map(
                |mut old_player_session| {
                    if let Some(new_player_session) =
                        new_player_sessions
                            .clone()
                            .into_iter()
                            .find(|new_player_session| {
                                new_player_session.steam_id == old_player_session.steam_id
                            })
                    {
                        if old_player_session.game_session_id == new_player_session.game_session_id
                        {
                            old_player_session.ended_at = chrono::Utc::now().naive_utc();
                            old_player_session.kills = new_player_session.kills;
                            old_player_session.perk = new_player_session.perk;
                            Some(old_player_session)
                        } else {
                            Some(new_player_session)
                        }
                    } else {
                        None
                    }
                },
            ));
        } else {
            updated_player_sessions.extend(new_player_sessions);
        }
        Ok(updated_player_sessions)
    }

    pub(crate) async fn log_player_sessions(&mut self) -> Result<(), Box<dyn Error>> {
        let player_sessions = self.create_new_players_sessions().await?;
        if player_sessions.is_empty() {
            self.player_sessions = None;
            return Ok(());
        }
        let updated_player_sessions = self.update_player_sessions(player_sessions).await?;
        self.player_sessions = Some(
            self.db_connection
                .log_player_sessions(updated_player_sessions)
                .await?,
        );
        Ok(())
    }
}
