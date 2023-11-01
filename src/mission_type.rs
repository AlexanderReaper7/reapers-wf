use procmacros::{Display, FromStr};
use serde::{Serialize, Deserialize};

use crate::filters::Filter;


#[derive(Debug,Serialize,Deserialize,Clone,Copy,PartialEq,Eq,Hash,Display,FromStr)]
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
    Hive,
    Assassination,
    Rush,
    Pursuit,
    Deception,
    Crossfire,
}
impl Filter for MissionType {
    fn apply_filter(&self, value: &crate::models::Fissure) -> bool {
        value.mission_type == *self
    }
}
