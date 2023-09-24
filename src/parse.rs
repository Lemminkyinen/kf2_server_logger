// use crate::model::Player;
use reqwest::header::HeaderMap;
use scraper::{ElementRef, Html, Selector};
use std::{
    collections::HashMap,
    error::Error,
    net::{IpAddr, Ipv4Addr},
};

use crate::model::{Perk, Player, PlayerInGame, PlayerInfo};

pub struct DocumentExtractor {
    document: Html,
}

impl DocumentExtractor {
    pub fn new(document: &str) -> Self {
        Self {
            document: Html::parse_document(document),
        }
    }

    pub fn parse_form_token(&self) -> Result<String, Box<dyn Error>> {
        let selector = Selector::parse(r#"input[name="token"]"#)?;
        let token = self
            .document
            .select(&selector)
            .next()
            .ok_or("Token not found")?;
        let value = token
            .value()
            .attr("value")
            .ok_or("Token value field not found")?;
        Ok(value.to_string())
    }

    fn parse_player_table(&self) -> Result<Vec<scraper::element_ref::ElementRef>, Box<dyn Error>> {
        let table_selector = Selector::parse(r#"table[id="players"]"#)?;
        let tbody_selector = Selector::parse("tbody")?;
        let tr_selector = Selector::parse("tr")?;
        let em_selector = Selector::parse("em")?;
        let player_table = self
            .document
            .select(&table_selector)
            .next()
            .ok_or(r#"table[id="players"] not found"#)?;
        let player_tbody = player_table
            .select(&tbody_selector)
            .next()
            .ok_or("tbody not found")?;
        let player_trs: Vec<ElementRef<'_>> = player_tbody.select(&tr_selector).collect();

        match player_trs
            .iter()
            .any(|e| e.select(&em_selector).next().is_some())
        {
            true => Ok({
                log::debug!("Player table contains em element, which means there are no players");
                Vec::new()
            }),
            false => Ok(player_trs),
        }
    }

    fn parse_tr_players<T>(
        tr_players: Vec<scraper::element_ref::ElementRef>,
        f: &dyn Fn(ElementRef) -> Result<T, Box<dyn Error>>,
    ) -> Vec<T>
    where
        T: Player,
    {
        tr_players
            .into_iter()
            .map(|tr_player| match f(tr_player) {
                Ok(player) => Some(player),
                Err(e) => {
                    log::error!("{}", e);
                    None
                }
            })
            .filter_map(|player| player)
            .collect()
    }

    pub fn parse_in_game_player_info(&self) -> Vec<PlayerInGame> {
        fn parse_tr_player(tr_player: ElementRef) -> Result<PlayerInGame, Box<dyn Error>> {
            let td_selector = Selector::parse("td")?;
            let mut td_fields = tr_player.select(&td_selector).skip(1);
            if td_fields.clone().count() != 7 {
                return Err("Wrong number of fields".into());
            }
            let name = td_fields.next().ok_or("Name tr not found")?.inner_html();
            let perk = Perk::map(&td_fields.next().ok_or("Perk tr not found")?.inner_html())?;
            let dosh = td_fields
                .next()
                .ok_or("Dosh tr not found")?
                .inner_html()
                .parse()
                .unwrap_or(0);
            let kills = td_fields
                .next()
                .ok_or("Kills tr not found")?
                .inner_html()
                .parse()
                .unwrap_or(0);
            let health = td_fields
                .next()
                .ok_or("Health tr not found")?
                .inner_html()
                .parse()
                .unwrap_or(0);
            let ping = td_fields
                .next()
                .ok_or("Ping tr not found")?
                .inner_html()
                .parse()?;
            let admin = td_fields
                .next()
                .ok_or("Admin tr not found")?
                .inner_html()
                .to_lowercase()
                == "yes";
            Ok(PlayerInGame {
                name,
                perk,
                dosh,
                kills,
                health,
                ping,
                admin,
            })
        }

        let tr_players = match self.parse_player_table() {
            Ok(tr_players) => tr_players,
            Err(e) => {
                log::error!("Error when parsing in game player table: {}", e);
                return Vec::new();
            }
        };

        Self::parse_tr_players(tr_players, &parse_tr_player)
    }

    pub fn parse_steam_player_info(&self) -> Vec<PlayerInfo> {
        fn parse_tr_player(tr_player: ElementRef) -> Result<PlayerInfo, Box<dyn Error>> {
            let td_selector = Selector::parse("td")?;
            let mut td_fields = tr_player.select(&td_selector).skip(1);
            if td_fields.clone().count() != 9 {
                return Err("Wrong number of fields".into());
            }
            let name = td_fields.next().ok_or("Name tr not found")?.inner_html();
            let ping = td_fields
                .next()
                .ok_or("Ping tr not found")?
                .inner_html()
                .parse()?;
            let ip = IpAddr::V4(
                td_fields
                    .next()
                    .ok_or("Ip tr not found")?
                    .inner_html()
                    .parse::<Ipv4Addr>()?,
            );
            let unique_net_id = td_fields
                .next()
                .ok_or("Unique Net ID tr not found")?
                .inner_html();
            let steam_id = td_fields
                .next()
                .ok_or("Steam ID tr not found")?
                .inner_html()
                .parse()?;
            let admin = td_fields
                .next()
                .ok_or("Admin tr not found")?
                .inner_html()
                .to_lowercase()
                == "yes";
            Ok(PlayerInfo {
                name,
                ping,
                ip,
                unique_net_id,
                steam_id,
                admin,
            })
        }

        let tr_players = match self.parse_player_table() {
            Ok(tr_players) => tr_players,
            Err(e) => {
                log::error!("Error when parsing steam player table: {}", e);
                return Vec::new();
            }
        };

        Self::parse_tr_players(tr_players, &parse_tr_player)
    }

    pub fn parse_current_map_info(&self) -> Result<String, Box<dyn Error>> {
        todo!()
    }

    pub fn parse_summary_info(&self) -> Result<String, Box<dyn Error>> {
        todo!()
    }
}

#[derive(Debug)]
pub struct HeaderExtractor {
    headers: HeaderMap,
    cookies: HashMap<String, String>,
}

impl HeaderExtractor {
    pub fn new(headers: HeaderMap) -> Self {
        let cookies = Self::parse_cookies(headers.clone());
        Self { headers, cookies }
    }

    fn parse_cookies(headers: HeaderMap) -> HashMap<String, String> {
        match headers.get("Set-Cookie") {
            Some(cookie) => match cookie.to_str() {
                Ok(cookie_str) => cookie_str
                    .split(';')
                    .into_iter()
                    .filter_map(|c| {
                        if let Some((name, value)) = c.trim().split_once('=') {
                            Some((name.to_owned(), value.to_owned()))
                        } else {
                            None
                        }
                    })
                    .collect(),
                Err(e) => {
                    log::error!("Error when parsing cookie: {}", e);
                    HashMap::new()
                }
            },
            None => HashMap::new(),
        }
    }

    pub fn get_cookie(&self, cookie: &str) -> Option<String> {
        self.cookies.get(cookie).cloned()
    }
}
