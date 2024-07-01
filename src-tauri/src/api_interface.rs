use std::time::Duration;

use reqwest::{self, ClientBuilder};
use anyhow::{anyhow, Result};
use serde::{Serialize, Deserialize};

use tynkerbase_universal::{
    crypt_utils::BinaryPacket, 
    proj_utils,
    constants::TYB_APIKEY_HTTP_HEADER,
};

pub async fn ping(endpoint: String) -> Result<()> {
    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true) // Disable TLS certificate validation
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    let _res = client
        .get(format!("{}", endpoint))
        .send()
        .await
        .map_err(|e| anyhow!("Error sending https request: {e}"))?;

    Ok(())
}

pub async fn get_id(endpoint: &str, tyb_key: &str) -> Result<String> {
    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true) 
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    let res = client
        .get(format!("{}", endpoint))
        .send()
        .await
        .map_err(|e| anyhow!("Error sending https request: {e}"))?;

    let id = res.text().await
        .map_err(|e| anyhow!("Error extracting text from response -> {}", e))?;

    Ok(id)
}

pub async fn create_proj(endpoint: &str, name: &str, data: Vec<u8>) -> Result<()> {
    // TODO: Add API key to this

    let endpoint = format!("{}/proj/create-proj?name={}", endpoint, name);

    let client = reqwest::Client::new();
    let res = client.post(endpoint)
        .body(data)
        .send()
        .await?;
    
    let res = res.bytes().await.unwrap();
    let res = res.to_vec();

    #[derive(Debug, Serialize, Deserialize)]
    enum AgentResponse {
        Ok(String),
        OkData(Vec<u8>),
        AgentErr(String),
        ClientErr(String),
    }

    let res: AgentResponse = bincode::deserialize(&res).unwrap();

    println!("{:?}", res);

    Ok(())
}

pub async fn transfer_files(endpoint: String, name: &str, file_dir: &str, tyb_key: &str) -> Result<()> {
    let v = vec![];
    let files = proj_utils::FileCollection::load(file_dir, &v)?;

    let packet = BinaryPacket::from(&files)?;
    let packet_bin = bincode::serialize(&packet)
        .map_err(|e| anyhow!("Error serializing binary packet: {}", e))?;

    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true) // Disable TLS certificate validation
        .timeout(Duration::from_secs(5))
        .build()?;

    let _res = client
        .post(format!("{}/files/proj/add-files-to-proj?name={}", endpoint, name))
        .body(packet_bin)
        .header(TYB_APIKEY_HTTP_HEADER, tyb_key)
        .send()
        .await
        .map_err(|e| anyhow!("Error sending https request: {e}"))?;

    Ok(())
}

pub async fn list_projects(endpoint: &str, tyb_key: &str) -> Result<Vec<String>> {
    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true) // Disable TLS certificate validation
        .build()?;

    let res = client
        .get(format!("{}/files/proj/list_projects", endpoint))
        .header(TYB_APIKEY_HTTP_HEADER, tyb_key)
        .send()
        .await
        .map_err(|e| anyhow!("Error sending https request: {e}"))?;

    let body = res.bytes().await
        .map_err(|e| anyhow!("Error extracting bytes from response -> {}", e))?;
    
    let projects: Vec<String> = bincode::deserialize(&body)
        .map_err(|e| anyhow!("Error deserializing data -> {}", e))?;

    Ok(projects)
}

pub async fn build_img (endpoint: &str, name: &str, tyb_key: &str) -> Result<()> {
    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true) // Disable TLS certificate validation
        .build()?;

    let _res = client
        .get(format!("{}/docker/proj/build-img?name={}", endpoint, name))
        .header(TYB_APIKEY_HTTP_HEADER, tyb_key)
        .send()
        .await
        .map_err(|e| anyhow!("Error sending https request: {e}"))?;

    Ok(())
}

pub async fn spawn_container(endpoint: &str, name: &str, tyb_key: &str) -> Result<()> {
    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true) // Disable TLS certificate validation
        .build()?;

    let _res = client
        .get(format!("{}/docker/proj/spawn-container?name={}", endpoint, name))
        .header(TYB_APIKEY_HTTP_HEADER, tyb_key)
        .send()
        .await
        .map_err(|e| anyhow!("Error sending https request: {e}"))?;

    Ok(())
}