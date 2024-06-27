use serde::{Serialize, Deserialize};
use bincode;
use anyhow::{anyhow, Result};
use std::{
    fs, 
    env,
    path::Path
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalState {
    pub email: Option<String>,
    pub pass_sha384: Option<String>,
    pub tyb_key: Option<String>,
    pub nodes: Vec<String>,
    pub projects: Vec<String>
}

impl GlobalState {
    pub fn load (email: &str, path: &str) -> Result<Self> {
        let mut res = GlobalState::default();
        res.email = Some(email.to_string());

        let full_path = res.get_path(path);

        Self::load_from_path(&full_path)
    }

    pub fn load_from_path(full_path: &str) -> Result<Self> {
        if !Path::new(full_path).exists() {
            return Err(anyhow!("Path doesn't exist"));
        }

        let obj = fs::read(full_path)
            .map_err(|e| anyhow!("Failed to read file: {e}"))?;

        let res = bincode::deserialize(&obj)
            .map_err(|e| anyhow!("Failed to deserialize data: {e}"))?;

        Ok(res)
    }

    pub fn save(&mut self, path: &str) -> Result<()> {
        if self.email.is_none() {
            return Err(anyhow!("Cannot save user info without an email"));
        }

        let path = self.get_path(path);

        let mut curr = GlobalState::load_from_path(&path)?;
        self.merge_from(curr).unwrap();

        let data = bincode::serialize(&self)
            .map_err(|e| anyhow!("Failed to serialize data: {e}"))?;

        fs::write(path, data)
            .map_err(|e| anyhow!("Failed to save data to disk: {e}"))?;

        Ok(())
    }

    fn get_path(&self, path: &str) -> String {
        let name = self.email.clone().unwrap();
        let name = name.replace("@", "AT-SYMBOL");
        format!("{}/{}.bin", path, name)
    }

    fn merge_from (&mut self, mut other: Self) -> Result<()> {
        if self.email != other.email {
            return Err(anyhow!("Emails don't match"));
        }
        if self.pass_sha384 != other.pass_sha384 {
            return Err(anyhow!("Passwords don't match"));
        }
        if self.tyb_key != other.tyb_key {
            return Err(anyhow!("Keys don't match"));
        }

        let mut other = other.clone();

        self.nodes.append(&mut other.nodes);
        self.projects.append(&mut other.projects);


        Ok(())
    }
}














