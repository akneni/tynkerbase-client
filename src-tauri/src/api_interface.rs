use reqwest;
use anyhow::{anyhow, Result};
use serde::{Serialize, Deserialize};

pub async fn create_proj(endpoint: &str, name: &str, data: Vec<u8>) -> Result<()> {
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

pub async fn get_pub_key(endpoint: &str) -> Result<Vec<u8>> {
    let endpoint = format!("{}/security/get-pub-key", &endpoint);
    let res = reqwest::get(&endpoint)
        .await
        .map_err(|e| anyhow!("{}", e))?;

    let res = res.bytes().await.unwrap();

    Ok(res.to_vec())
}