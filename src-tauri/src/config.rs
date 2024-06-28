use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub proj_name: String,
    pub node_names: Vec<String>,
    pub port_mapping: Vec<[u16; 2]>,
    pub volume_mapping: Vec<[String; 2]>,
}

