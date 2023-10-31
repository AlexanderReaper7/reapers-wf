use std::error::Error;

use crate::models::*;

const API_STR: &str = "https://api.warframestat.us/pc/";

pub async fn get_fissures() -> Result<Vec<Fissure>, Box<dyn Error>> {
    let fissures = reqwest::get(&format!("{}{}", API_STR, "fissures"))
        .await?.json::<Vec<Fissure>>().await?;
    Ok(fissures)
}
