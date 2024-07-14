use dirs::home_dir;
use std::{
    path::PathBuf,
    env::consts::OS,
};

pub fn app_data() -> String {
    if let Some(home_path) = home_dir() {
        let mut path = PathBuf::from(home_path);

        if OS == "linux" {
            path.push(".local/share/tynkerbase/");
        }
        else if OS == "windows" {
            path.push("AppData\\Local\\TynkerBase");
        }
        else {
            panic!("Error, OS `{}` not supported.", OS);
        }

        return path.to_str().unwrap().to_string();
    } 
    panic!("could not find home dir");
}

pub const AUTH_ENDPOINT: &str = "https://tynkerbase-server.shuttleapp.rs";
pub const PROJ_JSON_CONFIG: &str = "tynkerbase-config.json";
pub const NG_SKIP_WARN: &str = "ngrok-skip-browser-warning";