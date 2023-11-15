use std::{error::Error, fmt::Display};

use crate::{
    filters::{ExclusivityFilter, Factions, Tier},
    mission_type::MissionType,
    models::Fissure,
    util::comma_separated_string,
};

const DEFAULT_CONFIG: &str = include_str!("../default-config.toml");
pub const CONFIG_PATH: &str = "reapers-wf-config.toml";

#[derive(serde::Deserialize)]
pub struct Config {
    pub mission_filter: Vec<MissionType>,
    pub tier_filter: Vec<Tier>,
    pub faction_filter: Vec<Factions>,
    /// Whether to include, exclude or exclusively filter for void storms
    pub void_storm_filter: ExclusivityFilter,
    /// How often to refresh the fissure list in seconds
    pub refresh_rate: u64,
    /// How long before the fissure expires to send a notification in seconds
    pub time_before_expiry_notification: u64,
}
impl Config {
    pub async fn create_default_file() -> Result<(), Box<dyn Error>> {
        tokio::fs::write(CONFIG_PATH, DEFAULT_CONFIG).await?;
        Ok(())
    }
    pub async fn load() -> Result<Config, Box<dyn Error>> {
        let config = tokio::fs::read_to_string(CONFIG_PATH).await?;
        let conf = toml::from_str::<Config>(&config)?;
        Ok(conf)
    }
    pub fn apply_filters_cloned<'a>(&self, fissures: &Vec<Fissure>) -> Vec<Fissure> {
        fissures
            .iter()
            .filter(|fissure| {
                self.mission_filter
                    .iter()
                    .any(|filter| filter.apply_filter(fissure))
                    && self
                        .tier_filter
                        .iter()
                        .any(|filter| filter.apply_filter(fissure))
                    && self
                        .faction_filter
                        .iter()
                        .any(|filter| filter.apply_filter(fissure))
                    && self.void_storm_filter.apply_filter(fissure.is_storm)
            })
            .cloned()
            .collect::<Vec<Fissure>>()
    }

    pub fn apply_filters<'a>(&self, fissures: &'a Vec<Fissure>) -> Vec<&'a Fissure> {
        fissures
            .iter()
            .filter(|fissure| {
                self.mission_filter
                    .iter()
                    .any(|filter| filter.apply_filter(fissure))
                    && self
                        .tier_filter
                        .iter()
                        .any(|filter| filter.apply_filter(fissure))
                    && self
                        .faction_filter
                        .iter()
                        .any(|filter| filter.apply_filter(fissure))
                    && self.void_storm_filter.apply_filter(fissure.is_storm)
            })
            .collect::<Vec<&'a Fissure>>()
    }
}
impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Refresh Rate: {}s", self.refresh_rate)?;
        writeln!(
            f,
            "Time Before Expiry Notification: {}s",
            self.time_before_expiry_notification
        )?;
        writeln!(
            f,
            "Tier Filter: {}",
            comma_separated_string(&self.tier_filter)
        )?;
        writeln!(
            f,
            "Mission Filter: {}",
            comma_separated_string(&self.mission_filter)
        )?;
        writeln!(
            f,
            "Faction Filter: {}",
            comma_separated_string(&self.faction_filter)
        )?;
        write!(f, "Void Storm Filter: {}", self.void_storm_filter)?;
        Ok(())
    }
}
impl Default for Config {
    fn default() -> Self {
        toml::from_str(DEFAULT_CONFIG)
            .expect("Error parsing default config, default should always be valid")
    }
}
