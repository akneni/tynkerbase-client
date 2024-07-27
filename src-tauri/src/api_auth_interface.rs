use tynkerbase_universal::{
    self, 
    crypt_utils::hash_utils,
    netwk_utils::Node,
};

use crate::consts::AUTH_ENDPOINT;
use crate::agent_interface::validate_response;
use reqwest;
use bincode;
use anyhow::{anyhow, Result};

pub async fn login(email: &str, password: &str) -> Result<String> {
    let pass_sha256 = hash_utils::sha256(password);

    let endpoint = format!("{}/auth/login?email={}&pass_sha256={}", AUTH_ENDPOINT, email, pass_sha256);

    let res = reqwest::get(&endpoint)
        .await
        .map_err(|e| anyhow!("error sending get request: {e}"))?;
    let res = validate_response(res).await?;
    
    let salt = res.text().await
        .map_err(|e| anyhow!("error extracting test from response: {e}"))?;

    if salt.contains("Incorrect password") {
        return Err(anyhow!("Incorrect authentication"));
    }

    let pass_sha384 = hash_utils::sha384(password);
    let api_key = tynkerbase_universal::crypt_utils::gen_apikey(&pass_sha384, &salt);

    Ok(api_key)
}

pub async fn create_account(email: &str, password: &str) -> Result<()> {
    let pass_sha256 = hash_utils::sha256(password);

    let endpoint = format!("{}/auth/create-account?email={}&pass_sha256={}", AUTH_ENDPOINT, email, pass_sha256);

    let res = reqwest::get(&endpoint)
        .await
        .map_err(|e| anyhow!("error sending get request: {e}"))?;

    validate_response(res).await?;
    Ok(())
}

pub async fn delete_account(email: &str, password: &str) -> Result<()> {
    let pass_sha256 = hash_utils::sha256(password);

    let endpoint = format!("{}/auth/delete-account?email={}&pass_sha256={}", AUTH_ENDPOINT, email, pass_sha256);

    let res = reqwest::get(&endpoint)
        .await
        .map_err(|e| anyhow!("error sending get request: {e}"))?;

    validate_response(res).await?;
    Ok(())
}

pub async fn get_nodes(email: &str, pass_sha256: &str) -> Result<Vec<Node>>{
    let endpoint = format!("{}/ngrok/get-all-addrs?email={}&pass_sha256={}", AUTH_ENDPOINT, email, pass_sha256);

    let res = reqwest::get(&endpoint)
        .await
        .map_err(|e| anyhow!("Failed to communicate with server -> {}", e))?;

    let res = validate_response(res).await?;

    let bin = res
        .bytes()
        .await
        .map_err(|e| anyhow!("Failed to communicate with server -> {}", e))?
        .to_vec();

    bincode::deserialize(&bin)
        .map_err(|e| anyhow!("Failed to deserialize response -> {}", e))
}

pub async fn remove_node(email: &str, pass_sha256: &str, node_id: &str) -> Result<()> {
    let endpoint = format!("{}/ngrok/remove-addr?email={}&pass_sha256={}&node_id={}", AUTH_ENDPOINT, email, pass_sha256, node_id);
    let res = reqwest::get(&endpoint)
        .await
        .map_err(|e| anyhow!("Failed to communicate with server -> {}", e))?;

    validate_response(res).await?;
    Ok(())
}