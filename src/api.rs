use crate::models::*;

const API_STR: &str = "https://api.warframestat.us/pc/";

pub async fn get_fissures() -> reqwest::Result<Vec<Fissure>> {
    let fissures = reqwest::get(&format!("{}{}", API_STR, "fissures"))
        .await?.json::<Vec<Fissure>>().await?;
    Ok(fissures)
}
