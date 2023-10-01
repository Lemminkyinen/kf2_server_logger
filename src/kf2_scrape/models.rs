use std::{error::Error, net::IpAddr};

#[derive(Debug)]
pub enum Perk {
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
    pub fn map(input: &str) -> Result<Perk, Box<dyn Error>> {
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

pub struct Kf2WebPlayer {
    pub name: String,
    pub steam_id: u64,
}

#[derive(Debug)]
pub struct PlayerInGame {
    pub name: String,
    pub perk: Perk,
    pub dosh: u32,
    pub health: u32,
    pub kills: u32,
    pub ping: u32,
    pub admin: bool,
}

#[derive(Debug, Clone)]
pub struct PlayerInfo {
    pub name: String,
    pub ping: u32,
    pub ip: IpAddr,
    pub unique_net_id: String,
    pub steam_id: u64,
    pub admin: bool,
}

pub trait Player {}

impl Player for PlayerInfo {}

impl Player for PlayerInGame {}
