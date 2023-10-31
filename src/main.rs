use std::{
    error::Error,
    time::{Duration, Instant},
};

use models::Fissure;
use notify_rust::Notification;
use time::OffsetDateTime;
use tokio;

mod api;
mod mission_type;
mod models;
mod util;

use mission_type::MissionType;
use util::*;

const DEFAULT_CONFIG: &str = include_str!("../default-config.toml");
const CONFIG_PATH: &str = "reapers-wf-config.toml";

#[derive(serde::Deserialize)]
struct Config {
    mission_filter: Vec<MissionType>,
    tier_filter: Vec<u8>,
    refresh_rate: u64,
}
impl Config {
    fn create_default_file() -> Result<Config, Box<dyn Error>> {
        std::fs::write(CONFIG_PATH, DEFAULT_CONFIG)?;
        Ok(toml::from_str::<Config>(DEFAULT_CONFIG)?)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // load config
    let config = if let Ok(config) = std::fs::read_to_string(CONFIG_PATH) {
        if let Ok(conf) = toml::from_str::<Config>(&config) {
            conf
        } else {
            println!("Error parsing config file, if you have edited it, please fix it, otherwise delete it and restart the program.");
            println!("Press any key to exit");
            // pause for input so the user can read the error
            let mut _out = String::new();
            std::io::stdin().read_line(&mut _out)?;
            return Ok(());
        }
    } else {
        Config::create_default_file()?
    };
    // print the config
    println!("Starting fissure watcher with config:");
    println!("Refresh Rate: {}s", config.refresh_rate);
    println!("Tier Filter: {}", config.tier_filter.iter().map(|tier| format!("{}", tier)).collect::<Vec<String>>().join(", "));
    println!("Mission Filter: {}", config.mission_filter.iter().map(|mission| format!("{}", mission)).collect::<Vec<String>>().join(", "));


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
                    config.mission_filter.contains(&fissure.mission_type)
                        && config.tier_filter.contains(&fissure.tier_num)
                })
                .collect::<Vec<&models::Fissure>>();

            if filtered_fissures.len() == 0 {
                println!("No fissures found (of {}, {} new)", fissures.len(), new_count);
            } else {
                println!("{} New Fissure(s) (of {}, {} new):", filtered_fissures.len(), fissures.len(), new_count);
                let mut table_data = filtered_fissures
                .iter()
                .map(|fissure| fissure.table_string())
                .collect::<Vec<Vec<String>>>();
                table_data.insert(0, Fissure::table_headers());
                println!(
                    "{}",
                    table(table_data)
                );
                Notification::new()
                    .summary("New Fissures")
                    .body(
                        filtered_fissures
                            .iter()
                            .map(|fissure| fissure.to_string())
                            .collect::<Vec<String>>().join("\n").as_str()
                    )
                    .show()?;
            }
        }
        tokio::time::sleep(Duration::from_secs(config.refresh_rate) - start.elapsed()).await
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
            // dbg!(fissure.clone());
            old.push(fissure);
            new_count += 1;
        }
    }
    Ok(new_count)
}
