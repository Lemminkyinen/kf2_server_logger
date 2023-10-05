use diesel::deserialize::FromSql;
use diesel::expression::AsExpression;
use diesel::mysql::Mysql;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::VarChar;
use std::io::Write;
use std::{error::Error, net::IpAddr};

#[derive(Debug, Clone)]
pub(crate) enum Perk {
    Berserker,
    Commando,
    Support,
    FieldMedic,
    Demolitionist,
    Firebug,
    Gunslinger,
    Sharpshooter,
    Survivalist,
    Swat,
}

impl Perk {
    pub(super) fn map(input: &str) -> Result<Perk, Box<dyn Error>> {
        let mut input = input.to_lowercase();
        input.retain(|c| !c.is_whitespace());
        match input.as_str() {
            "berserker" => Ok(Perk::Berserker),
            "commando" => Ok(Perk::Commando),
            "support" => Ok(Perk::Support),
            "fieldmedic" => Ok(Perk::FieldMedic),
            "demolitionist" => Ok(Perk::Demolitionist),
            "firebug" => Ok(Perk::Firebug),
            "gunslinger" => Ok(Perk::Gunslinger),
            "sharpshooter" => Ok(Perk::Sharpshooter),
            "survivalist" => Ok(Perk::Survivalist),
            "swat" => Ok(Perk::Swat),
            e => Err(format!("Perk {} not found", e).into()),
        }
    }
}

impl ToString for Perk {
    fn to_string(&self) -> String {
        match self {
            Perk::Berserker => "Berserker".to_string(),
            Perk::Commando => "Commando".to_string(),
            Perk::Support => "Support".to_string(),
            Perk::FieldMedic => "Field Medic".to_string(),
            Perk::Demolitionist => "Demolitionist".to_string(),
            Perk::Firebug => "Firebug".to_string(),
            Perk::Gunslinger => "Gunslinger".to_string(),
            Perk::Sharpshooter => "Sharpshooter".to_string(),
            Perk::Survivalist => "Survivalist".to_string(),
            Perk::Swat => "Swat".to_string(),
        }
    }
}

pub struct Kf2WebPlayer {
    pub name: String,
    pub steam_id: u64,
}

#[derive(Debug)]
pub(crate) struct PlayerInGame {
    pub(crate) name: String,
    pub(crate) perk: Perk,
    pub(crate) dosh: u32,
    pub(crate) health: u32,
    pub(crate) kills: u32,
    pub(crate) ping: u32,
    pub(crate) admin: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct PlayerInfo {
    pub(crate) name: String,
    pub(crate) ping: u32,
    pub(crate) ip: IpAddr,
    pub(crate) unique_net_id: String,
    pub(crate) steam_id: u64,
    pub(crate) admin: bool,
}

pub trait Player {}

impl Player for PlayerInfo {}

impl Player for PlayerInGame {}

#[derive(Debug, Clone)]
pub(crate) enum KfDifficulty {
    Normal,
    Hard,
    Suicidal,
    HellOnEarth,
}

impl KfDifficulty {
    pub(super) fn map(input: &str) -> Result<Self, Box<dyn Error>> {
        match input {
            "Normal" => Ok(KfDifficulty::Normal),
            "Hard" => Ok(KfDifficulty::Hard),
            "Suicidal" => Ok(KfDifficulty::Suicidal),
            "Hell on Earth" => Ok(KfDifficulty::HellOnEarth),
            _ => Err(format!("Unknown difficulty {}", input).into()),
        }
    }
}

impl ToString for KfDifficulty {
    fn to_string(&self) -> String {
        match self {
            KfDifficulty::Normal => "Normal".to_string(),
            KfDifficulty::Hard => "Hard".to_string(),
            KfDifficulty::Suicidal => "Suicidal".to_string(),
            KfDifficulty::HellOnEarth => "Hell on Earth".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct GameInfo {
    pub(crate) max_waves: u16,
    pub(crate) current_wave: u16,
    pub(crate) max_players: u16,
    pub(crate) current_players: u16,
    pub(crate) map_name: String,
    pub(crate) difficulty: KfDifficulty,
    pub(crate) game_type: String,
    // pub(super) boss_name: String,
}
