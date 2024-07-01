use serde::{Serialize, Deserialize};
use bincode;
use anyhow::{anyhow, Result};
use std::{
    fs, 
    env,
    path::Path
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Node {
    pub name: String,
    pub id: String,
    pub ip_addr: Option<String>,
}


#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalState {
    pub email: Option<String>,
    pub pass_sha384: Option<String>,
    pub tyb_key: Option<String>,
    pub nodes: Vec<Node>,
    pub projects: Vec<String>
}

impl GlobalState {
    pub fn load (path: &str) -> Result<Self> {
        let bytes = fs::read(path)
            .map_err(|e| anyhow!("Error reading from file `{}` -> {}", path, e))?;
        let state: Self = bincode::deserialize(&bytes)
            .map_err(|e| anyhow!("Failed to deserialize state -> {}", e))?;
        Ok(state)
    }

    pub fn save(&self, path: &str) -> Result<()> {
        let bytes = bincode::serialize(&self)
            .map_err(|e| anyhow!("Error serializing state -> {}", e))?;
        fs::write(path, &bytes)
            .map_err(|e| anyhow!("Error saving state to file -> {}", e))?;

        Ok(())
    }
}














