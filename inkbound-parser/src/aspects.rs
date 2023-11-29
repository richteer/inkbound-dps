use serde::{Serialize, Deserialize};
use strum::EnumIter;

// TODO: Consider manually implementing (De)Serialize
//  May be more future proofed to not use a named string in a logfile,
//  and letting clients handle sorting that part out.
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, EnumIter, Hash, Ord, PartialOrd)]
pub enum Aspect {
    MagmaMiner,
    Mosscloak,
    Clairvoyant,
    Weaver,
    Obelisk,
    StarCaptain,
    Chainbreaker,
    Godkeeper,
    // Possibly not yet implemented
    Unknown(String),
}

impl std::fmt::Display for Aspect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Aspect::MagmaMiner => "Magma Miner",
            Aspect::Mosscloak => "Mosscloak",
            Aspect::Clairvoyant => "Clairvoyent",
            Aspect::Weaver => "Weaver",
            Aspect::Obelisk => "Obelisk",
            Aspect::StarCaptain => "Star Captain",
            Aspect::Chainbreaker => "Chainbreaker",
            Aspect::Godkeeper => "Godkeeper",
            Aspect::Unknown(id) => id.as_str(),
        })
    }
}

impl Aspect {
    /// Get an Aspect enum from the internal string
    pub fn from_id(id: &str) -> Self {
        match id {
            "C01" => Aspect::MagmaMiner,
            "C02" => Aspect::Mosscloak,
            "C03" => Aspect::Clairvoyant,
            "C04" => Aspect::Weaver,
            "C05" => Aspect::Obelisk,
            "C06" => Aspect::Unknown("C06".to_string()),
            "C07" => Aspect::StarCaptain,
            "C08" => Aspect::Chainbreaker,
            "C09" => Aspect::Godkeeper,
            _ => Aspect::Unknown(id.to_string()),
        }
    }
}
