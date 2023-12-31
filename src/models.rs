use std::fmt::Display;
use serde::Deserialize;

use crate::filters::{Factions, Tier};

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct Fissure {
    /// unique identifier for this object/event/thing
    pub id:	String,
    /// ISO-8601 formatted timestamp for when the event began
    #[serde(with = "time::serde::iso8601")]
    pub activation:	time::OffsetDateTime,
    /// ISO-8601 formatted timestamp for when the event is expected to end
    #[serde(with = "time::serde::iso8601")]
    pub expiry:	time::OffsetDateTime,
    /// Short-time-formatted duration string representing the start of the event
    #[serde(rename = "startString")]
    pub start_string: String,
    /// Whether the event is currently active
    pub active:	bool,
    /// Node name with planet
    pub node: String,
    /// Whether the fissure is still present
    pub expired: bool,
    /// Short-formatted string estimating the time until the event/mission is closed
    pub eta: String,
    #[serde(rename = "missionType")]
    pub mission_type: super::MissionType,
    #[serde(rename = "missionKey")]
    pub mission_key: String,
    pub tier: Tier,
    /// Numeric tier corresponding to the tier
    #[serde(rename = "tierNum")]
    pub tier_num: u8,
    pub enemy: Factions,
    #[serde(rename = "enemyKey")]
    pub enemy_key: Factions,
    /// Whether this fissure is a void storm
    #[serde(rename = "isStorm")]
    pub is_storm: bool,
    /// Whether this fissure is on the Steel Path
    #[serde(rename = "isHard")]
    pub is_hard: bool,
}
impl Display for Fissure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{} {} on {}", if self.is_hard {"SP "} else {""}, self.tier, self.mission_type, self.node)
    }
}
impl Fissure {
    pub fn table_string(&self) -> Vec<String> {
        let time_format: Vec<time::format_description::FormatItem<'_>> = time::format_description::parse(
            "[hour]:[minute]:[second]").unwrap();
        vec![
            if self.is_hard {"SP".to_string()} else {"".to_string()},
            self.tier.to_string(),
            self.mission_type.to_string(),
            self.node.clone(),
            self.enemy.to_string(),
            self.expiry.format(&time_format).unwrap(),
        ]
    }
    pub fn table_headers() -> Vec<String> {
        vec![
            "SP".to_string(),
            "Tier".to_string(),
            "Mission Type".to_string(),
            "Node (Region)".to_string(),
            "Faction".to_string(),
            "Expiry".to_string(),
        ]
    }
}