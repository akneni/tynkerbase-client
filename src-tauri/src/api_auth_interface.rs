use tynkerbase_universal::{self, crypt_utils::hash_utils};
use reqwest;
use bincode;
use anyhow::{anyhow, Result};


const ENDPOINT: &str = "https://tynkerbase-server.shuttleapp.rs";

pub async fn login(email: &str, password: &str) -> Result<String> {
    let pass_sha256 = hash_utils::sha256(password);

    let endpoint = format!("{}/auth/login?email={}&pass_sha256={}", ENDPOINT, email, pass_sha256);

    let res = reqwest::get(&endpoint)
        .await
        .map_err(|e| anyhow!("error sending get request: {e}"))?;

    #[cfg(debug_assertions)] {
        println!("\n\nRESPONSE:\n{:#?}\n\n", &res);
    }

    let salt = res.text().await
        .map_err(|e| anyhow!("error extracting test from response: {e}"))?;

    #[cfg(debug_assertions)] {
        println!("\n\nRESPONSE TEXT:\n{:#?}\n\n", &salt);
    }

    let pass_sha384 = hash_utils::sha384(password);
    let api_key = tynkerbase_universal::crypt_utils::gen_apikey(&pass_sha384, &salt);

    Ok(api_key)
}

pub async fn create_account(email: &str, password: &str) -> Result<()> {
    let pass_sha256 = hash_utils::sha256(password);

    let endpoint = format!("{}/auth/create-account?email={}&pass_sha256={}", ENDPOINT, email, pass_sha256);

    let res = reqwest::get(&endpoint)
        .await
        .map_err(|e| anyhow!("error sending get request: {e}"))?;

    #[cfg(debug_assertions)] {
        println!("\n\nRESPONSE:\n{:#?}\n\n", &res);
    }
    
    Ok(())
}