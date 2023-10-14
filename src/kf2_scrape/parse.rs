use super::models::{GameInfo, KfDifficulty, Perk, PlayerData, PlayerInGame, PlayerInfo};
use log::error;
use reqwest::header::HeaderMap;
use scraper::{ElementRef, Html, Selector};
use std::{
    collections::HashMap,
    error::Error,
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
};

pub(super) struct ElementParse;
impl ElementParse {
    fn int<T>(element: Option<ElementRef>, err_msg: &str) -> Result<T, Box<dyn Error>>
    where
        T: FromStr,
    {
        let s = element.ok_or(err_msg)?.inner_html();
        if let Ok(int) = s.parse() {
            Ok(int)
        } else {
            let msg = format!("Parse int error. Cannot parse {}", s);
            Err(msg.into())
        }
    }

    fn bool(element: Option<ElementRef>, err_msg: &str) -> Result<bool, Box<dyn Error>> {
        let s = element.ok_or(err_msg)?.inner_html().to_lowercase();
        if s == "yes" {
            Ok(true)
        } else if s == "no" {
            Ok(false)
        } else {
            let msg = format!("Parse bool error. Cannot parse {} into boolean", s);
            Err(msg.into())
        }
    }

    fn string(element: Option<ElementRef>, err_msg: &str) -> Result<String, Box<dyn Error>> {
        Ok(element.ok_or(err_msg)?.inner_html())
    }

    fn ip_addr(element: Option<ElementRef>, err_msg: &str) -> Result<IpAddr, Box<dyn Error>> {
        Ok(IpAddr::V4(
            element.ok_or(err_msg)?.inner_html().parse::<Ipv4Addr>()?,
        ))
    }

    fn player_in_game<'a>(
        mut td_fields: impl Iterator<Item = ElementRef<'a>>,
    ) -> Result<PlayerInGame, Box<dyn Error>> {
        let name = ElementParse::string(td_fields.next(), "Name tr not found")?;
        let perk = Perk::map(&ElementParse::string(
            td_fields.next(),
            "Perk tr not found",
        )?)?;
        let dosh = ElementParse::int(td_fields.next(), "Dosh td not found").unwrap_or(0);
        let health = ElementParse::int(td_fields.next(), "Health td not found").unwrap_or(0);
        let kills = ElementParse::int(td_fields.next(), "Kills td not found").unwrap_or(0);
        let ping = ElementParse::int(td_fields.next(), "Ping td not found").unwrap_or(0);
        let admin = ElementParse::bool(td_fields.next(), "Admin td not found")?;
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

    fn player_info<'a>(
        mut td_fields: impl Iterator<Item = ElementRef<'a>>,
    ) -> Result<PlayerInfo, Box<dyn Error>> {
        let name = ElementParse::string(td_fields.next(), "Name td not found")?;
        let ping = ElementParse::int(td_fields.next(), "Ping td not found").unwrap_or(0);
        let ip = ElementParse::ip_addr(td_fields.next(), "IP td not found")?;
        let unique_net_id = ElementParse::string(td_fields.next(), "Unique Net ID td not found")?;
        let steam_id = ElementParse::int(td_fields.next(), "Steam ID td not found")?;
        td_fields.next();
        let admin = ElementParse::bool(td_fields.next(), "Admin td not found")?;
        Ok(PlayerInfo {
            name,
            ping,
            ip,
            unique_net_id,
            steam_id,
            admin,
        })
    }
}

pub(crate) struct DocumentExtractor {
    document: Html,
}

impl DocumentExtractor {
    pub(crate) fn new(document: &str) -> Self {
        Self {
            document: Html::parse_document(document),
        }
    }

    pub(crate) fn parse_form_token(&self) -> Result<String, Box<dyn Error>> {
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

    fn parse_player_table(&self) -> Result<Vec<ElementRef>, Box<dyn Error>> {
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

    fn parse_tr_player(tr_player: ElementRef) -> Result<PlayerData, Box<dyn Error>> {
        let td_selector = Selector::parse("td")?;
        let td_fields = tr_player.select(&td_selector).skip(1);
        let count = td_fields.clone().count();
        match count {
            7 => {
                let player_in_game = ElementParse::player_in_game(td_fields)?;
                Ok(PlayerData::PlayerInGame(player_in_game))
            }
            8 => {
                let player_info = ElementParse::player_info(td_fields)?;
                Ok(PlayerData::PlayerInfo(player_info))
            }
            n => {
                log::error!("Wrong number of fields in player table {}", n);
                Err(format!("Wrong number of fields in player table {}", n).into())
            }
        }
    }

    fn parse_tr_players(tr_players: Vec<ElementRef>) -> Vec<PlayerData> {
        tr_players
            .into_iter()
            .filter_map(|tr_player| match Self::parse_tr_player(tr_player) {
                Ok(player) => Some(player),
                Err(e) => {
                    log::error!("{}", e);
                    None
                }
            })
            .collect()
    }

    pub(crate) fn parse_in_game_player_info(&self) -> Vec<PlayerInGame> {
        let tr_players = match self.parse_player_table() {
            Ok(tr_players) => tr_players,
            Err(e) => {
                log::error!("Error when parsing in game player table: {}", e);
                return Vec::new();
            }
        };
        Self::parse_tr_players(tr_players)
            .into_iter()
            .filter_map(|player| player.into_p_in_game())
            .collect()
    }

    pub(crate) fn parse_steam_player_info(&self) -> Vec<PlayerInfo> {
        let tr_players = match self.parse_player_table() {
            Ok(tr_players) => tr_players,
            Err(e) => {
                log::error!("Error when parsing steam player table: {}", e);
                return Vec::new();
            }
        };

        Self::parse_tr_players(tr_players)
            .into_iter()
            .filter_map(|player| player.into_p_info())
            .collect()
    }

    fn parse_current_game(&self) -> Result<Vec<scraper::element_ref::ElementRef>, Box<dyn Error>> {
        let current_game_selector = Selector::parse(r#"dl[id="currentGame"]"#)?;
        let dd_selector = Selector::parse("dd")?;
        let current_game = &self
            .document
            .select(&current_game_selector)
            .next()
            .ok_or(r#"dl[id="currentGame"] not found"#)?;
        Ok(current_game.select(&dd_selector).collect())
    }

    fn parse_current_rules(&self) -> Result<Vec<scraper::element_ref::ElementRef>, Box<dyn Error>> {
        let current_rules_selector = Selector::parse(r#"dl[id="currentRules"]"#)?;
        let dd_selector = Selector::parse("dd")?;
        let current_rules = &self
            .document
            .select(&current_rules_selector)
            .next()
            .ok_or(r#"dl[id="currentRules"] not found"#)?;
        Ok(current_rules.select(&dd_selector).collect())
    }

    pub(crate) fn parse_current_map_info(&self) -> Result<GameInfo, Box<dyn Error>> {
        let current_game = self.parse_current_game()?;
        let current_rules = self.parse_current_rules()?;
        if current_game.is_empty() || current_rules.is_empty() {
            error!(
                "current game length: {}, current rules length: {}",
                current_game.len(),
                current_rules.len()
            );
            return Err("Current game or rules not found".into());
        }
        let game_type = current_game[2].inner_html();
        let map_name = current_game[3].inner_html();
        let wave = current_rules[0].inner_html();
        let difficulty = KfDifficulty::map(&current_rules[1].inner_html())?;
        let players = current_rules[2].inner_html();

        let (current_wave, max_wave) = wave
            .split_once('/')
            .ok_or("Wave does not contain char '/'")?;
        let (current_players, max_players) = players
            .split_once('/')
            .ok_or("Wave does not contain char '/'")?;

        let current_wave = current_wave.parse()?;
        let max_waves = max_wave.parse()?;
        let current_players = current_players.parse()?;
        let max_players = max_players.parse()?;

        Ok(GameInfo {
            max_waves,
            current_wave,
            max_players,
            current_players,
            map_name,
            difficulty,
            game_type,
        })
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

#[cfg(test)]
mod tests_element_parse {
    use super::*;
    use scraper::{Html, Selector};

    fn get_document_with(text: &str) -> Html {
        let doc = format!(
            r#"<!DOCTYPE html>
               <meta charset="utf-8">
               <table>
               <tr>
                <td>{text}</td>
                </tr>
               </table>
               <h1 class="foo">Hello, <i>world!</i></h1>"#,
            text = text
        );
        Html::parse_document(&doc)
    }

    fn get_element_ref<'a>(html: &'a Html, selector: &str) -> ElementRef<'a> {
        let selector = Selector::parse(selector).unwrap();
        html.select(&selector).next().unwrap()
    }

    #[test]
    fn test_parse_custom_err() {
        let e = None;
        let r = ElementParse::int::<u32>(e, "This error is from space");
        assert!(r.is_err());
        let e = r.unwrap_err().to_string();
        assert_eq!(e, "This error is from space");
    }

    #[test]
    fn test_parse_int_u32() {
        let max = u32::MAX.to_string();
        let html = get_document_with(&max);
        let e = Some(get_element_ref(&html, "td"));
        let r = ElementParse::int::<u32>(e, "");
        assert_eq!(r.unwrap(), 4294967295);
    }

    #[test]
    fn test_parse_int_u32_zero() {
        let html = get_document_with("0");
        let e = Some(get_element_ref(&html, "td"));
        let r = ElementParse::int::<u32>(e, "");
        assert_eq!(r.unwrap(), 0);
    }

    #[test]
    fn test_parse_int_u32_error() {
        let html = get_document_with("123123Kissa");
        let e = Some(get_element_ref(&html, "td"));
        let r = ElementParse::int::<u32>(e, "");
        assert!(r.is_err());
        let e = r.unwrap_err();
        assert!(e
            .to_string()
            .contains("Parse int error. Cannot parse 123123Kissa"));
    }

    #[test]
    fn test_parse_int_u16() {
        let max = u16::MAX.to_string();
        let html = get_document_with(&max);
        let e = Some(get_element_ref(&html, "td"));
        let r = ElementParse::int::<u16>(e, "");
        assert_eq!(r.unwrap(), 65535);
    }

    #[test]
    fn test_parse_int_u16_zero() {
        let html = get_document_with("0");
        let e = Some(get_element_ref(&html, "td"));
        let r = ElementParse::int::<u16>(e, "");
        assert_eq!(r.unwrap(), 0);
    }

    #[test]
    fn test_parse_int_u16_error() {
        let html = get_document_with("123123Kissa");
        let e = Some(get_element_ref(&html, "td"));
        let r = ElementParse::int::<u16>(e, "");
        assert!(r.is_err());
        let e = r.unwrap_err().to_string();
        assert_eq!(e, "Parse int error. Cannot parse 123123Kissa");
    }

    #[test]
    fn test_parse_bool_true() {
        let html = get_document_with("Yes");
        let e = Some(get_element_ref(&html, "td"));
        let r = ElementParse::bool(e, "");
        assert_eq!(r.unwrap(), true);
    }

    #[test]
    fn test_parse_bool_false() {
        let html = get_document_with("No");
        let e = Some(get_element_ref(&html, "td"));
        let r = ElementParse::bool(e, "");
        assert_eq!(r.unwrap(), false);
    }

    #[test]
    fn test_parse_bool_error() {
        let html = get_document_with("Kissa");
        let e = Some(get_element_ref(&html, "td"));
        let r = ElementParse::bool(e, "");
        assert!(r.is_err());
        let e = r.unwrap_err().to_string();
        assert_eq!(e, "Parse bool error. Cannot parse kissa into boolean");
    }

    #[test]
    fn test_parse_ip_addr() {
        let html = get_document_with("127.0.0.1");
        let e = Some(get_element_ref(&html, "td"));
        let r = ElementParse::ip_addr(e, "");
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        assert_eq!(r.unwrap(), ip);
    }

    #[test]
    fn test_parse_string() {
        let html = get_document_with("Hello, world!");
        let e = Some(get_element_ref(&html, "td"));
        let r = ElementParse::string(e, "");
        assert_eq!(r.unwrap(), "Hello, world!");
    }

    #[test]
    fn test_parse_player_in_game() {
        let doc = r#"
        <table id="players" class="grid" width="100%">
        <tbody>
        <tr class="even">
        <td style="background: transparent; color: transparent;">&#160;</td>
        <td>Kissa</td>
        <td>Demolitionist</td>
        <td class="right">460</td>
        <td class="right">83</td>
        <td class="right">86</td>
        <td class="right" title="Packet loss: ">36</td>
        <td class="center">No</td>
        </tr>
        <tr class="odd">
        <td style="background: transparent; color: transparent;">&#160;</td>
        <td>DeepBDarkBFantasy</td>
        <td>Field Medic</td>
        <td class="right">362</td>
        <td class="right">125</td>
        <td class="right">41</td>
        <td class="right" title="Packet loss: ">84</td>
        <td class="center">No</td>
        </tr>
        </tbody>
        </table>"#;
        let html = Html::parse_fragment(doc);
        let e = Some(get_element_ref(&html, "tr")).unwrap();
        let td_selector = Selector::parse("td").unwrap();
        let td_fields = e.select(&td_selector).skip(1);
        let r = ElementParse::player_in_game(td_fields).unwrap();
        assert_eq!(r.name, "Kissa");
        assert_eq!(r.perk.to_string(), "Demolitionist");
        assert_eq!(r.dosh, 460);
        assert_eq!(r.health, 83);
        assert_eq!(r.kills, 86);
        assert_eq!(r.ping, 36);
        assert_eq!(r.admin, false);
    }

    #[test]
    fn test_parse_player_info() {
        let doc = r#"
        <table id="players" class="grid" width="100%">
        <tbody>
        <tr class="even">
        <td style="background: transparent; color: transparent;">&#160;</td>
        <td>Gooby</td>
        <td class="right">123</td>
        <td>127.0.0.1</td>
        <td>0x0110000</td>
        <td>76561198</td>
        <td></td>
        <td class="center">Yes</td>
        <td class="center">Yes</td>
        </tr>
        </tbody>
        </table>"#;
        let html = Html::parse_fragment(doc);
        let e = Some(get_element_ref(&html, "tr")).unwrap();
        let td_selector = Selector::parse("td").unwrap();
        let td_fields = e.select(&td_selector).skip(1);
        let r = ElementParse::player_info(td_fields).unwrap();
        assert_eq!(r.name, "Gooby");
        assert_eq!(r.ping, 123);
        assert_eq!(r.ip, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        assert_eq!(r.unique_net_id, "0x0110000");
        assert_eq!(r.steam_id, 76561198);
        assert_eq!(r.admin, true);
    }
}

#[cfg(test)]
mod tests_header_extractor {
    use super::*;
    use reqwest::header::HeaderValue;

    #[test]
    fn test_header_extractor() {
        let mut headers = HeaderMap::new();
        headers.append(
            "Set-Cookie",
            HeaderValue::from_static("kissa=koira;hevonen=mammutti;"),
        );
        let r = HeaderExtractor::new(headers);
        let ck1 = r.get_cookie("kissa").unwrap();
        let ck2 = r.get_cookie("hevonen").unwrap();
        assert_eq!(ck1, "koira");
        assert_eq!(ck2, "mammutti");
    }
}

#[cfg(test)]
mod tests_document_extractor {
    use super::*;

    fn get_form_token_document(token: &str) -> String {
        format!(
            r#"<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.1//EN" "http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd">
        <html xmlns="http://www.w3.org/1999/xhtml" xml:lang="en">
        <body class="">
        </div>
        <div id="gamesummary">
        </div>
        <div id="messages">
        </div>
        <div id="content">
        <form id="loginform" method="post" action="/ServerAdmin/" autocomplete="off">
        <fieldset>
        <legend>Login</legend>
        <div class="section">
        <input type="hidden" name="token" value="{token}" />
        <input type="hidden" id="password_hash" name="password_hash" value="" />
        <dl>
            <dt><label for="username">Username</label></dt>
            <dd><input type="text" id="username" name="username" value="" /></dd>
            <dt><label for="password">Password</label></dt>
            <dd><input type="password" id="password" name="password" value="" /></dd>
            <dt><label for="remember" title="Duration of inactivity before you need to log in again.">Remember</label></dt>
            <dd><select name="remember">
                <option value="0">Until next map load</option>
                <option value="-1" selected="selected">Browser session</option>
                <!-- Number of seconds -->
                <option value="1800">30 minutes</option>
                <option value="3600">1 hour</option>
                <option value="86400">1 day</option>
                <option value="604800">1 week</option>
                <option value="2678400">1 month</option>
            </select></dd>
            <dd><button type="submit">login</button></dd>
        </dl>
        </div>
        </fieldset>
        </form>
        </div>
        <div id="footer">
        Copyright 2014 Tripwire Interactive LLC
        &#8212;
        <a href="/ServerAdmin/about">About the Killing Floor 2 WebAdmin</a>
        </div>
        </body>
        </html>
        "#,
            token = token
        )
    }

    fn get_player_table_document(players: bool) -> String {
        format!(
            r#"<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.1//EN" "http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd">
            <html xmlns="http://www.w3.org/1999/xhtml" xml:lang="en">
            <body class="">
                <div id="gamesummary">
                    <h2>Current Game</h2>
                    <script type="text/javascript" src="/images/gamesummary.js?gzip"></script>
                </div>
                <div id="messages">
                </div>
                <div id="content">
                    <fieldset id="notesField"
                        title="Here you can leave some notes. They will be stored on the server so other administrators can see and edit them.">
                        <legend>Notes</legend>
                    </fieldset>
                    <table width="100%" id="currentinfo">
                        <tr>
                            <td>
                                <h3>Players</h3>
                                <div class="section narrow">
                                    <table id="players" class="grid" width="100%">
                                        <thead>
                                            <tr>
                                                <th>&#160;</th>
                                                <th><a href="/ServerAdmin/current/info?sortby=name&amp;reverse="
                                                        class="sortable ">Name</a></th>
                                                <th><a href="/ServerAdmin/current/info?sortby=perk&amp;reverse="
                                                        class="sortable ">Perk</th>
                                                <th><a href="/ServerAdmin/current/info?sortby=score&amp;reverse="
                                                        class="sortable sorted">Dosh</a>
                                                </th>
                                                <th>Health</th>
                                                <th><a href="/ServerAdmin/current/info?sortby=kills&amp;reverse="
                                                        class="sortable ">Kills</a></th>
                                                <th><a href="/ServerAdmin/current/info?sortby=ping&amp;reverse="
                                                        class="sortable ">Ping</a></th>
                                                <th>Admin</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {players}
                                        </tbody>
                                    </table>
                                </div>
                            </td>
                        </tr>
                    </table>
                </div>
            </body>
            </html>"#,
            players = if players {
                String::from(
                    r#"
            <tr class="even">
                <td style="background: transparent; color: transparent;">&#160;</td>
                <td>koira</td>
                <td>Demolitionist</td>
                <td class="right">460</td>
                <td class="right">83</td>
                <td class="right">86</td>
                <td class="right" title="Packet loss: ">36</td>
                <td class="center">No</td>
            </tr>
            <tr class="odd">
                <td style="background: transparent; color: transparent;">&#160;</td>
                <td>DeepBDarkBFantasy</td>
                <td>Field Medic</td>
                <td class="right">362</td>
                <td class="right">125</td>
                <td class="right">41</td>
                <td class="right" title="Packet loss: ">84</td>
                <td class="center">No</td>
            </tr>
            <tr class="even">
                <td style="background: transparent; color: transparent;">&#160;</td>
                <td>WhiteHex</td>
                <td>Berserker</td>
                <td class="right">936</td>
                <td class="right">200</td>
                <td class="right">42</td>
                <td class="right" title="Packet loss: ">52</td>
                <td class="center">No</td>
            </tr>
            <tr class="odd">
                <td style="background: transparent; color: transparent;">&#160;</td>
                <td>` CRÆZY</td>
                <td>Demolitionist</td>
                <td class="right">0</td>
                <td class="right"></td>
                <td class="right">0</td>
                <td class="right" title="Packet loss: ">0</td>
                <td class="center">No</td>
            </tr>"#,
                )
            } else {
                String::from("")
            }
        )
    }

    fn get_steam_player_table_document(players: bool) -> String {
        format!(
            r#"<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.1//EN" "http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd">
            <html xmlns="http://www.w3.org/1999/xhtml" xml:lang="en">
            <body class="">
                <div id="gamesummary">
                    <h2>Current Game</h2>
                    <script type="text/javascript" src="/images/gamesummary.js?gzip"></script>
                </div>
                <div id="messages">
                </div>
                <div id="content">
                    <fieldset id="notesField"
                        title="Here you can leave some notes. They will be stored on the server so other administrators can see and edit them.">
                        <legend>Notes</legend>
                    </fieldset>
                    <table width="100%" id="currentinfo">
                        <tr>
                            <td>
                                <h3>Players</h3>
                                <div class="section narrow">
                                    <table id="players" class="grid" width="100%">
                                        <thead>
                                            <tr>
                                                <th>&#160;</th>
                                                <th><a href="/ServerAdmin/current/info?sortby=name&amp;reverse="
                                                        class="sortable ">Name</a></th>
                                                <th><a href="/ServerAdmin/current/info?sortby=perk&amp;reverse="
                                                        class="sortable ">Perk</th>
                                                <th><a href="/ServerAdmin/current/info?sortby=score&amp;reverse="
                                                        class="sortable sorted">Dosh</a>
                                                </th>
                                                <th>Health</th>
                                                <th><a href="/ServerAdmin/current/info?sortby=kills&amp;reverse="
                                                        class="sortable ">Kills</a></th>
                                                <th><a href="/ServerAdmin/current/info?sortby=ping&amp;reverse="
                                                        class="sortable ">Ping</a></th>
                                                <th>Admin</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {players}
                                        </tbody>
                                    </table>
                                </div>
                            </td>
                        </tr>
                    </table>
                </div>
            </body>
            </html>"#,
            players = if players {
                String::from(
                    r#"
            <tr class="even">
                <td style="background: transparent; color: transparent;">&#160;</td>
                <td>koira</td>
                <td>123</td>
                <td class="right">127.0.0.1</td>
                <td class="right">asdasd123</td>
                <td class="right">123123123</td>
                <td class="center"></td>
                <td class="center">No</td>
                <td class="center"></td>
            </tr>
            <tr class="odd">
                <td style="background: transparent; color: transparent;">&#160;</td>
                <td>Gooby</td>
                <td class="right">0</td>
                <td>192.168.1.100</td>
                <td>0x01100001049DE279</td>
                <td>76561198037721721</td>
                <td></td>
                <td class="center">Yes</td>
                <td class="center">Yes</td>
            </tr>
            <tr class="even">
                <td style="background: transparent; color: transparent;">&#160;</td>
                <td>WhiteHex</td>
                <td>123</td>
                <td class="right">127.0.0.1</td>
                <td class="right">asdasd123</td>
                <td class="right">123123123</td>
                <td class="center"></td>
                <td class="center">No</td>
                <td class="center"></td>
            </tr>
            <tr class="odd">
                <td style="background: transparent; color: transparent;">&#160;</td>
                <td>` CRÆZY</td>
                <td></td>
                <td class="right">127.0.0.1</td>
                <td class="right">asdasd123</td>
                <td class="right">123123123</td>
                <td class="center"></td>
                <td class="center">No</td>
                <td class="center"></td>
            </tr>"#,
                )
            } else {
                String::from("")
            }
        )
    }

    #[test]
    fn test_parse_token() {
        let document = get_form_token_document("kissa123");
        let extractor = DocumentExtractor::new(&document);
        let token = extractor.parse_form_token().unwrap();
        assert_eq!(token, "kissa123");
    }

    #[test]
    fn test_parse_token_empty() {
        let document = get_form_token_document("");
        let extractor = DocumentExtractor::new(&document);
        let token = extractor.parse_form_token().unwrap();
        assert_eq!(token, "");
    }

    #[test]
    fn test_parse_token_no_token() {
        let document = get_player_table_document(true);
        let extractor = DocumentExtractor::new(&document);
        let token = extractor.parse_form_token();
        assert!(token.is_err());
    }

    #[test]
    fn test_parse_player_table() {
        let document = get_player_table_document(true);
        let extractor = DocumentExtractor::new(&document);
        let players = extractor.parse_player_table().unwrap();
        assert!(!players.is_empty());
        assert!(players.len() == 4);
    }

    #[test]
    fn test_parse_player_table_empty() {
        let document = get_player_table_document(false);
        let extractor = DocumentExtractor::new(&document);
        let players = extractor.parse_player_table().unwrap();
        assert!(players.is_empty());
    }

    #[test]
    fn test_parse_in_game_player_info() {
        let document = get_player_table_document(true);
        let extractor = DocumentExtractor::new(&document);
        let players = extractor.parse_in_game_player_info();
        assert!(players.len() == 4);
        assert!(players[0].name == "koira");
        assert!(players[0].perk == Perk::Demolitionist);
        assert!(players[0].dosh == 460);
        assert!(players[0].health == 83);
        assert!(players[0].kills == 86);
        assert!(players[0].ping == 36);
        assert!(players[0].admin == false);
        assert!(players.last().unwrap().name == "` CRÆZY");
        assert!(players.last().unwrap().kills == 0);
    }

    #[test]
    fn test_parse_in_game_player_info_empty() {
        let document = get_player_table_document(false);
        let extractor = DocumentExtractor::new(&document);
        let players = extractor.parse_in_game_player_info();
        assert!(players.is_empty());
    }

    #[test]
    fn test_parse_steam_player_info() {
        env_logger::init();
        let document = get_steam_player_table_document(true);
        let extractor = DocumentExtractor::new(&document);
        let players = extractor.parse_steam_player_info();
        assert!(players.len() == 4);
        assert!(players[0].name == "koira");
        assert!(players[0].ping == 123);
        assert!(players[0].ip == IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        assert!(players[0].unique_net_id == "asdasd123");
        assert!(players[0].steam_id == 123123123);
        assert!(players[0].admin == false);
        assert!(players.last().unwrap().name == "` CRÆZY");
        assert!(players.last().unwrap().ping == 0);
    }

    #[test]
    fn test_parse_steam_player_info_empty() {
        let document = get_steam_player_table_document(false);
        let extractor = DocumentExtractor::new(&document);
        let players = extractor.parse_steam_player_info();
        assert!(players.is_empty());
    }
}
