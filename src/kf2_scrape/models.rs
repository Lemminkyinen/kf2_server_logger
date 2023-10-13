use std::{error::Error, net::IpAddr};

#[derive(Debug, Clone, PartialEq, Eq)]
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
    NotSelected,
}

impl Perk {
    pub(super) fn map(input: &str) -> Result<Self, Box<dyn Error>> {
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
            "" => Ok(Perk::NotSelected),
            _ => Err("Invalid perk".into()),
        }
    }
}

impl ToString for Perk {
    fn to_string(&self) -> String {
        match self {
            Perk::Berserker => String::from("Berserker"),
            Perk::Commando => String::from("Commando"),
            Perk::Support => String::from("Support"),
            Perk::FieldMedic => String::from("Field Medic"),
            Perk::Demolitionist => String::from("Demolitionist"),
            Perk::Firebug => String::from("Firebug"),
            Perk::Gunslinger => String::from("Gunslinger"),
            Perk::Sharpshooter => String::from("Sharpshooter"),
            Perk::Survivalist => String::from("Survivalist"),
            Perk::Swat => String::from("Swat"),
            Perk::NotSelected => String::from("Not Selected"),
        }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum KfDifficulty {
    Normal,
    Hard,
    Suicidal,
    HellOnEarth,
}

impl KfDifficulty {
    pub(super) fn map(input: &str) -> Result<Self, Box<dyn Error>> {
        let mut input = input.to_lowercase();
        input.retain(|c| !c.is_whitespace());
        match input.as_str() {
            "normal" => Ok(KfDifficulty::Normal),
            "hard" => Ok(KfDifficulty::Hard),
            "suicidal" => Ok(KfDifficulty::Suicidal),
            "hellonearth" => Ok(KfDifficulty::HellOnEarth),
            _ => Err(format!("Unknown difficulty {}", input).into()),
        }
    }
}

impl ToString for KfDifficulty {
    fn to_string(&self) -> String {
        match self {
            KfDifficulty::Normal => String::from("Normal"),
            KfDifficulty::Hard => String::from("Hard"),
            KfDifficulty::Suicidal => String::from("Suicidal"),
            KfDifficulty::HellOnEarth => String::from("Hell on Earth"),
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

#[cfg(test)]
mod tests_perk {
    use super::*;

    #[test]
    fn test_perk_map_() {
        let perk = Perk::map("Berserker").unwrap();
        assert_eq!(perk, Perk::Berserker);

        let perk = Perk::map("Field Medic").unwrap();
        assert_eq!(perk, Perk::FieldMedic);
    }

    #[test]
    fn test_perk_map_error() {
        let perk = Perk::map("kissa");
        assert!(perk.is_err());
    }

    #[test]
    fn test_perk_empty() {
        let perk = Perk::map("").unwrap();
        assert_eq!(perk, Perk::NotSelected);
    }

    #[test]
    fn test_perk_to_string() {
        let perk = Perk::FieldMedic;
        assert_eq!(perk.to_string(), "Field Medic");

        let perk = Perk::Swat;
        assert_eq!(perk.to_string(), "Swat");

        let perk = Perk::NotSelected;
        assert_eq!(perk.to_string(), "Not Selected");
    }
}

#[cfg(test)]
mod tests_kf_difficulty {
    use super::*;

    #[test]
    fn test_kf_difficulty_map_() {
        let difficulty = KfDifficulty::map("Normal").unwrap();
        assert_eq!(difficulty, KfDifficulty::Normal);

        let difficulty = KfDifficulty::map("Hell on Earth").unwrap();
        assert_eq!(difficulty, KfDifficulty::HellOnEarth);
    }

    #[test]
    fn test_kf_difficulty_map_error() {
        let difficulty = KfDifficulty::map("kissa");
        assert!(difficulty.is_err());
    }

    #[test]
    fn test_kf_difficulty_to_string() {
        let difficulty = KfDifficulty::Normal;
        assert_eq!(difficulty.to_string(), "Normal");

        let difficulty = KfDifficulty::HellOnEarth;
        assert_eq!(difficulty.to_string(), "Hell on Earth");

        let difficulty = KfDifficulty::Suicidal;
        assert_eq!(difficulty.to_string(), "Suicidal");
    }
}
