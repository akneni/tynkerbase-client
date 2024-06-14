use reqwest;
use anyhow::{anyhow, Result};


pub async fn create_proj(endpoint: &str) -> Result<()> {
    let res = reqwest::get(endpoint)
        .await
        .map_err(|e| anyhow!("{}", e))?;

    

    Ok(())
}