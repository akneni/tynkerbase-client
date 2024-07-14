use std::{process, time::Duration};

use reqwest::{self, ClientBuilder};
use anyhow::{anyhow, Result};
use serde::{Serialize, Deserialize};

use crate::consts::NG_SKIP_WARN;
use tynkerbase_universal::{
    constants::TYB_APIKEY_HTTP_HEADER, crypt_utils::{compression_utils, BinaryPacket}, file_utils
};

pub async fn ping(endpoint: String) -> Result<()> {
    let endpoint = parse_endpoint(endpoint)?;
    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true) // Disable TLS certificate validation
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    let res = client
        .get(format!("{}", endpoint))
        .header(NG_SKIP_WARN, "easter egg here")
        .send()
        .await
        .map_err(|e| anyhow!("Error sending https request: {e}"))?;

    validate_response(res).await?;
    Ok(())
}

pub async fn get_id(endpoint: &str, tyb_key: &str) -> Result<String> {
    let endpoint = parse_endpoint(endpoint)?;
    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true) 
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    let res = client
        .get(format!("{}", endpoint))
        .header(TYB_APIKEY_HTTP_HEADER, tyb_key)
        .header(NG_SKIP_WARN, "easter egg here")
        .send()
        .await
        .map_err(|e| anyhow!("Error sending https request: {e}"))?;

    let res = validate_response(res).await?;
    let id = res.text().await
        .map_err(|e| anyhow!("Error extracting text from response -> {}", e))?;

    Ok(id)
}

pub async fn create_proj(endpoint: &str, name: &str, tyb_key: &str) -> Result<()> {
    let endpoint = parse_endpoint(endpoint)?;
    let endpoint = format!("{}/files/proj/create-proj?name={}", endpoint, name);
    #[cfg(debug_assertions)] println!("ENDPOINT: {}", endpoint);

    let res = reqwest::ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(5))
        .build()? 
        .get(endpoint)
        .header(TYB_APIKEY_HTTP_HEADER, tyb_key)
        .header(NG_SKIP_WARN, "easter egg here")
        .send()
        .await?;
    
    validate_response(res).await?;
    Ok(())
}

pub async fn transfer_files(endpoint: &str, name: &str, tyb_key: &str, files: &file_utils::FileCollection) -> Result<()> {
    let endpoint = parse_endpoint(endpoint)?;

    let mut packet = BinaryPacket::from(files)?;
    // compression_utils::compress_brotli(&mut packet)?;

    let packet_bin = bincode::serialize(&packet)
        .map_err(|e| anyhow!("Error serializing binary packet: {}", e))?;

    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true) // Disable TLS certificate validation
        .timeout(Duration::from_secs(10))
        .build()?;

    let res = client
        .post(format!("{}/files/proj/add-files-to-proj?name={}", endpoint, name))
        .body(packet_bin)
        .header(TYB_APIKEY_HTTP_HEADER, tyb_key)
        .header(NG_SKIP_WARN, "easter egg here")
        .send()
        .await
        .map_err(|e| anyhow!("Error sending https request: {e}"))?;

    validate_response(res).await?;
    Ok(())
}

pub async fn deploy_proj(endpoint: &str, name: &str, tyb_key: &str, files: &file_utils::FileCollection) -> Result<()> {
    create_proj(endpoint, name, tyb_key).await?;
    transfer_files(endpoint, name, tyb_key, files).await?;
    Ok(())
}

pub async fn list_projects(endpoint: &str, tyb_key: &str) -> Result<Vec<String>> {
    let endpoint = parse_endpoint(endpoint)?;

    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true) 
        .timeout(Duration::from_secs(5))
        .build()?;

    let res = client
        .get(format!("{}/files/proj/list-projects", endpoint))
        .header(TYB_APIKEY_HTTP_HEADER, tyb_key)
        .header(NG_SKIP_WARN, "easter egg here")
        .send()
        .await
        .map_err(|e| anyhow!("Error sending https request: {e}"))?;

    let res = validate_response(res).await?;
    let body = res.bytes().await
        .map_err(|e| anyhow!("Error extracting bytes from response -> {}", e))?;
    
    let projects: Vec<String> = bincode::deserialize(&body)
        .map_err(|e| anyhow!("Error deserializing data -> {}", e))?;

    Ok(projects)
}

pub async fn build_img (endpoint: &str, name: &str, tyb_key: &str) -> Result<()> {
    let endpoint = parse_endpoint(endpoint)?;

    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true) 
        .timeout(Duration::from_secs(5))
        .build()?;

    let res = client
        .get(format!("{}/docker/proj/build-img?name={}", endpoint, name))
        .header(TYB_APIKEY_HTTP_HEADER, tyb_key)
        .header(NG_SKIP_WARN, "easter egg here")
        .send()
        .await
        .map_err(|e| anyhow!("Error sending https request: {e}"))?;

    validate_response(res).await?;
    Ok(())
}

pub async fn spawn_container(endpoint: &str, name: &str, tyb_key: &str) -> Result<()> {
    let endpoint = parse_endpoint(endpoint)?;

    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true) 
        .timeout(Duration::from_secs(5))
        .build()?;

    let res = client
        .get(format!("{}/docker/proj/spawn-container?name={}", endpoint, name))
        .header(TYB_APIKEY_HTTP_HEADER, tyb_key)
        .header(NG_SKIP_WARN, "easter egg here")
        .send()
        .await
        .map_err(|e| anyhow!("Error sending https request: {e}"))?;

    validate_response(res).await?;
    Ok(())
}

fn parse_endpoint(endpoint: impl Into<String>) -> Result<String> {
    let endpoint: String = endpoint.into();
    if endpoint.starts_with("http") {
        return Ok(endpoint);
    }

    let is_ip = endpoint.len() < 16 && endpoint
        .chars()
        .all(|c| ".0123456789".contains(c));

    if is_ip {
        return Ok(format!("https://{}:7462", endpoint))
    }

    Err(anyhow!("Endpoint `{}` is in an unexpected format.", endpoint))
}

async fn validate_response(response: reqwest::Response) -> Result<reqwest::Response> {
    if !response.status().is_success() {
        let status = response.status();
        let mut text = response.text().await.unwrap_or("NONE".to_string());
        return Err(anyhow!("\nNon 200 response from node\nStatus Code: {:?}\nText Body: {}\n", status, text));
    }
    Ok(response)
}