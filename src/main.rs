use std::{
    error::Error,
    time::{Duration, Instant},
};

use models::Fissure;
use notify_rust::Notification;
use time::{Time, OffsetDateTime};
use tokio;

mod api;
mod mission_type;
mod models;
mod util;

use mission_type::MissionType;
use util::*;

struct Config {
    mission_filter: Vec<MissionType>,
    tier_filter: Vec<u8>,
    refresh_rate: Duration,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            mission_filter: vec![
                MissionType::Disruption,
            ],
            tier_filter: vec![1, 2, 3, 4],
            refresh_rate: Duration::from_secs(60 * 5),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::default();
    // print the config
    println!("Starting fissure watcher with config:");
    println!("Refresh Rate: {}s", config.refresh_rate.as_secs());
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
                println!("No fissures found (of {})", fissures.len());
            } else {
                println!("New Fissure(s):");
                println!(
                    "{}",
                    table(
                        filtered_fissures
                            .iter()
                            .map(|fissure| fissure.table_string())
                            .collect::<Vec<Vec<String>>>()
                    )
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
        tokio::time::sleep(config.refresh_rate - start.elapsed()).await
    }
}

/// Updates the given vector of Fissures with the current Fissures, returning a count of the new Fissures in the old vec
async fn update_fissures(old: &mut Vec<Fissure>) -> Result<u32, Box<dyn Error>> {
    let current = api::get_fissures().await?;

    // remove expired fissures
    let expired: Vec<usize> = old
        .iter()
        .enumerate()
        .filter(|(_, fissure)| !current.contains(&fissure))
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
