use std::{error::Error, time::Duration};

extern crate procmacros;

mod api;
mod filters;
mod mission_type;
mod models;
mod util;
mod config;
mod fissure_watcher;
mod app;
mod crossterm;
mod ui;

use mission_type::MissionType;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let tick_rate = Duration::from_millis(20);
    crate::crossterm::run(tick_rate).await?;
    Ok(())
}
