use std::{
    error::Error,
    time::{Duration, Instant},
};

use models::Fissure;
use notify_rust::Notification;
use time::OffsetDateTime;
use tokio;

extern crate procmacros;

mod api;
mod filters;
mod mission_type;
mod models;
mod util;

use filters::*;
use mission_type::MissionType;
use util::*;

const DEFAULT_CONFIG: &str = include_str!("../default-config.toml");
const CONFIG_PATH: &str = "reapers-wf-config.toml";

#[derive(serde::Deserialize)]
struct Config {
    mission_filter: Vec<MissionType>,
    tier_filter: Vec<Tier>,
    faction_filter: Vec<Factions>,
    /// Whether to include, exclude or exclusively filter for void storms
    void_storm_filter: ExclusivityFilter,
    /// How often to refresh the fissure list in seconds
    refresh_rate: u64,
    /// How long before the fissure expires to send a notification in seconds
    time_before_expiry_notification: u64,
}
impl Config {
    fn create_default_file() -> Result<Config, Box<dyn Error>> {
        std::fs::write(CONFIG_PATH, DEFAULT_CONFIG)?;
        Ok(toml::from_str::<Config>(DEFAULT_CONFIG)?)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // print application version with compile timestamp
    const BUILT_ON: &str = compile_time::datetime_str!();
    println!(
        "Reaper's Warframe Fissure Watcher v{} (compiled {})",
        env!("CARGO_PKG_VERSION"),
        BUILT_ON.replace(|c| { c == 'T' || c == 'Z' }, " ")
    );
    // load config
    let config = if let Ok(config) = std::fs::read_to_string(CONFIG_PATH) {
        if let Ok(conf) = toml::from_str::<Config>(&config) {
            conf
        } else {
            println!("!!! Error parsing config file, if you have edited it, please fix it, otherwise delete it and restart the program.");
            println!("Press any key to exit");
            // pause for input so the user can read the error
            let mut _out = String::new();
            std::io::stdin().read_line(&mut _out)?;
            return Ok(());
        }
    } else {
        println!(
            "!!! No config file found, creating default config file {}.",
            CONFIG_PATH
        );
        Config::create_default_file()?
    };
    // print the config
    println!("Starting fissure watcher with config:");
    println!("Refresh Rate: {}s", config.refresh_rate);
    println!(
        "Tier Filter: {}",
        comma_separated_string(&config.tier_filter)
    );
    println!(
        "Mission Filter: {}",
        comma_separated_string(&config.mission_filter)
    );
    println!(
        "Faction Filter: {}",
        comma_separated_string(&config.faction_filter)
    );
    println!("Void Storm Filter: {}", config.void_storm_filter);

    let mut fissures: Vec<models::Fissure> = Vec::new();
    // refresh every time refresh_rate elapses
    loop {
        let start = Instant::now();
        let new_count = update_fissures(&mut fissures).await?;
        let format = time::format_description::parse("[hour]:[minute]:[second]")?;
        print!("[{}] ", OffsetDateTime::now_utc().time().format(&format)?);
        if new_count == 0 {
            println!("No new fissures");
        } else {
            let filtered_fissures = fissures
                .iter()
                .skip(fissures.len() - new_count as usize)
                .filter(|fissure| {
                    config
                        .mission_filter
                        .iter()
                        .any(|filter| filter.apply_filter(fissure))
                        && config
                            .tier_filter
                            .iter()
                            .any(|filter| filter.apply_filter(fissure))
                        && config
                            .faction_filter
                            .iter()
                            .any(|filter| filter.apply_filter(fissure))
                        && config.void_storm_filter.apply_filter(fissure.is_storm)
                })
                .collect::<Vec<&models::Fissure>>();

            if filtered_fissures.len() == 0 {
                println!(
                    "No fissures found (of {}, {} new)",
                    fissures.len(),
                    new_count
                );
            } else {
                println!(
                    "{} New Fissure(s) (of {}, {} new):",
                    filtered_fissures.len(),
                    fissures.len(),
                    new_count
                );
                // print table
                let mut table_data = filtered_fissures
                    .iter()
                    .map(|fissure| fissure.table_string())
                    .collect::<Vec<Vec<String>>>();
                table_data.insert(0, Fissure::table_headers());
                println!("{}", table(table_data));
                // send notification
                Notification::new()
                    .summary("New Fissures")
                    .body(
                        filtered_fissures
                            .iter()
                            .map(|fissure| fissure.to_string())
                            .collect::<Vec<String>>()
                            .join("\n")
                            .as_str(),
                    )
                    .show()?;
                // enqueue notification for expiry
                for fissure in filtered_fissures {
                    spawn_expiry_notification(fissure, config.time_before_expiry_notification);
                }
            }
        }
        tokio::time::sleep(Duration::from_secs(config.refresh_rate) - start.elapsed()).await
    }
}

fn spawn_expiry_notification(fissure: &Fissure, time_before_expiry_notification: u64) {
    let expiry = fissure.expiry - Duration::from_secs(time_before_expiry_notification);
    let now = OffsetDateTime::now_utc();
    if expiry > now {
        let duration = expiry - now;
        let sleep_time = std::time::Duration::from_secs_f64(duration.as_seconds_f64());
        let fissure_str = fissure.to_string();
        tokio::spawn(async move {
            tokio::time::sleep(sleep_time).await;
            Notification::new()
                .summary(
                    format!(
                        "Fissure is Expiring In {} Seconds",
                        time_before_expiry_notification
                    )
                    .as_str(),
                )
                .body(fissure_str.as_str())
                .show().unwrap();
        });
    }
}

/// Updates the given vector of Fissures with the current Fissures, returning a count of the new Fissures
async fn update_fissures(old: &mut Vec<Fissure>) -> Result<u32, Box<dyn Error>> {
    let current = api::get_fissures().await?;
    // remove expired fissures
    let expired: Vec<usize> = old
        .iter()
        .enumerate()
        .filter(|(_, fissure)| current.iter().find(|f| f.id == fissure.id).is_none())
        .map(|(i, _)| i)
        .collect();
    for i in expired.into_iter().rev() {
        old.remove(i);
    }
    // add new fissures
    let mut new_count = 0;
    for fissure in current {
        if old
            .iter()
            .find(|old_fissure| old_fissure.id == fissure.id)
            .is_none()
        {
            old.push(fissure);
            new_count += 1;
        }
    }
    Ok(new_count)
}
