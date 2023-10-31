use std::{fmt::{Display, Formatter}, str::FromStr};
use serde::{Serialize, Deserialize};
use crate::util;

#[derive(Debug,Serialize,Deserialize,Clone,Copy,PartialEq,Eq,Hash)]
pub enum MissionType {
    Capture,
    Defense,
    Excavation,
    Extermination,
    Interception,
    #[serde(rename = "Mobile Defense")]
    MobileDefense,
    Rescue,
    Sabotage,
    Survival,
    Spy,
    Hijack,
    Assault,
    Defection,
    #[serde(rename = "Infested Salvage")]
    InfestedSalvage,
    Disruption,
    #[serde(rename = "Sanctuary Onslaught")]
    SanctuaryOnslaught,
    #[serde(rename = "Free Roam")]
    FreeRoam,
    Arena,
    Skirmish,
    Orphix,
    Volatile,
}
impl Display for MissionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", util::separate_camel_case(&format!("{:?}", self)))
    }
}
impl FromStr for MissionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Capture" => Ok(MissionType::Capture),
            "Defense" => Ok(MissionType::Defense),
            "Excavation" => Ok(MissionType::Excavation),
            "Extermination" => Ok(MissionType::Extermination),
            "Interception" => Ok(MissionType::Interception),
            "MobileDefense" | "Mobile Defense" => Ok(MissionType::MobileDefense),
            "Rescue" => Ok(MissionType::Rescue),
            "Sabotage" => Ok(MissionType::Sabotage),
            "Survival" => Ok(MissionType::Survival),
            "Spy" => Ok(MissionType::Spy),
            "Hijack" => Ok(MissionType::Hijack),
            "Assault" => Ok(MissionType::Assault),
            "Defection" => Ok(MissionType::Defection),
            "InfestedSalvage" | "Infested Salvage" => Ok(MissionType::InfestedSalvage),
            "Disruption" => Ok(MissionType::Disruption),
            "SanctuaryOnslaught" | "Sanctuary Onslaught" => Ok(MissionType::SanctuaryOnslaught),
            "FreeRoam" | "Free Roam" => Ok(MissionType::FreeRoam),
            "Arena" => Ok(MissionType::Arena),
            "Skirmish" => Ok(MissionType::Skirmish),
            "Orphix" => Ok(MissionType::Orphix),
            "Volatile" => Ok(MissionType::Volatile),
            _ => Err(format!("Unknown mission type: {}", s)),
        }
    }
}
