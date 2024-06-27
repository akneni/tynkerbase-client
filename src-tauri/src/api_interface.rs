use reqwest::{self, ClientBuilder};
use anyhow::{anyhow, Result};
use serde::{Serialize, Deserialize};

use tynkerbase_universal::{
    crypt_utils::BinaryPacket, 
    proj_utils,
    constants::TYB_APIKEY_HTTP_HEADER,
};


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

pub async fn transfer_files(endpoint: &str, name: &str, file_dir: &str, tyb_key: &str) -> Result<()> {
    let v = vec![];
    let files = proj_utils::FileCollection::load(file_dir, &v)?;

    let packet = BinaryPacket::from(&files)?;
    let packet_bin = bincode::serialize(&packet)
        .map_err(|e| anyhow!("Error serializing binary packet: {}", e))?;

    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true) // Disable TLS certificate validation
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