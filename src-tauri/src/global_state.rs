use tynkerbase_universal::{crypt_utils::hash_utils, netwk_utils::Node};
use crate::consts;
use crate::api_auth_interface::get_nodes;
use serde::{Serialize, Deserialize};
use bincode;
use anyhow::{anyhow, Result};
use tokio::runtime::Runtime;
use std::{
    fs,
    path::Path
};



#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalState {
    pub email: String,
    pub password: String,
    pub tyb_key: String,
    pub nodes: Vec<Node>,
    pub projects: Vec<String>
}

impl GlobalState {
    pub fn new(email: &str, password: &str, tyb_key: &str) -> Self {
        GlobalState {
            email: email.to_string(), 
            password: password.to_string(),
            tyb_key: tyb_key.to_string(),
            nodes: vec![],
            projects: vec![],
        }
    }

    pub fn path() -> String {
        format!("{}/global-state.bin", consts::app_data())
    }

    pub fn exists() -> bool {
        let path: String = format!("{}/global-state.bin", consts::app_data());
        Path::new(&path).exists()
    }

    pub fn load () -> Result<Self> {
        let path: String = format!("{}/global-state.bin", consts::app_data());
        let bytes = fs::read(&path)
            .map_err(|e| anyhow!("Error reading from file `{}` -> {}", &path, e))?;
        let state: Self = bincode::deserialize(&bytes)
            .map_err(|e| anyhow!("Failed to deserialize state -> {}", e))?;
        Ok(state)
    }

    pub fn save(&self) -> Result<()> {
        let app_data = consts::app_data();
        if !Path::new(&app_data).exists() {
            fs::create_dir_all(&app_data).unwrap();
        }
        let path: String = format!("{}/global-state.bin", &app_data);

        let bytes = bincode::serialize(&self)
            .map_err(|e| anyhow!("Error serializing state -> {}", e))?;
        fs::write(&path, &bytes)
            .map_err(|e| anyhow!("Error saving state to file -> {}", e))?;

        Ok(())
    }

    pub fn populate_nodes(&mut self) -> Result<()> {
        let rt = Runtime::new().unwrap();
        let pass_sha256 = hash_utils::sha256(&self.password);

        let f = get_nodes(&self.email, &pass_sha256);
        let mut nodes = rt.block_on(f)?;
        self.nodes.append(&mut nodes);
        Ok(())
    }
}














