use std::{
    error::Error,
    fmt::Display,
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
mod config;

use filters::*;
use mission_type::MissionType;
use util::*;
use config::Config;
use std::{io, thread};
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Widget, Block, Borders},
    layout::{Layout, Constraint, Direction},
    Terminal
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
#[tokio::main]
async fn oldmain() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        let size = f.size();
        let block = Block::default()
            .title("Block")
            .borders(Borders::ALL);
        f.render_widget(block, size);
    })?;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    // print application version with compile timestamp
    const BUILT_ON: &str = compile_time::datetime_str!();
    println!(
        "Reaper's Warframe Fissure Watcher v{} (compiled {})",
        env!("CARGO_PKG_VERSION"),
        BUILT_ON.replace('T', " ").replace('Z', "")
    );
    // load config
    let config = Config::load()?;
    // print the config
    println!("Starting fissure watcher with config:");
    println!("{}", config);
    let mut fissures: Vec<Fissure> = Vec::new();
    // refresh every time refresh_rate elapses
    loop {
        let start = Instant::now();
        let new_count = update_fissures(&mut fissures).await?;
        let format = time::format_description::parse("[hour]:[minute]:[second]")?;
        print!("[{}] ", OffsetDateTime::now_utc().time().format(&format)?);
        if new_count == 0 {
            println!("No new fissures");
        } else {
            let filtered_fissures = config.apply_filters(&fissures);

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
                .show()
                .unwrap();
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
