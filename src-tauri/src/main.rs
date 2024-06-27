// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


#![allow(unused)] // TEMPORARY, REMOVE BEFORE PROD  

mod api_interface;
mod api_auth_interface;
mod constants;
mod global_state;

use tynkerbase_universal::{
    crypt_utils::{
        self, compression_utils, hash_utils, BinaryPacket, CompressionType
    }, 
    docker_utils, 
    proj_utils::{self, FileCollection}
};

use std::fs;
use std::env;
use std::sync::Mutex;
use std::io::{self, Write};
use rpassword::read_password;
use bincode;
use tokio;
use clap::{Parser, Subcommand};

use constants::ACCT_INFO;
use global_state::GlobalState;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str, state: tauri::State<'_, Mutex<GlobalState>>) -> String {
    format!("Hello, {}! You current path is", name)
}

fn launch_gui(state: GlobalState) {
    let state = Mutex::new(state);

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}



#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: TopLevelCmds,
}

#[derive(Subcommand)]
enum TopLevelCmds {
    Gui,
    Login,
    Build { 
        filename: String, 
        #[arg(short='O', long="optim", default_value_t = 0)]
        optimization_level: u8, 
    },
}


fn main() {
    let cmds = Cli::parse();
    let mut gstate = match env::var("TYB_CURR_SESSION_EMAIL") {
        Ok(e) => GlobalState::load(&e, ACCT_INFO).unwrap(),
        _ => GlobalState::default(),
    };
    let rt = tokio::runtime::Runtime::new().unwrap();

    match cmds.command {
        TopLevelCmds::Gui => {
            launch_gui(gstate);
        },
        TopLevelCmds::Login => {
            let email = prompt("Enter your email: ");
            let pass = prompt_password("Enter your password: ");

            let key = api_auth_interface::login(&email, &pass);
            let key = match rt.block_on(key) {
                    Ok(r) => r,
                    Err(e) => {
                        println!("Error logging in: {e}");
                        return;
                    }
            };
            env::set_var("TYB_CURR_SESSION_EMAIL", &email);
            let mut gstate = GlobalState::default();
            gstate.email = Some(email);
            gstate.pass_sha384 = Some(hash_utils::sha384(&pass));
            gstate.tyb_key = Some(key);
            gstate.save(ACCT_INFO);
        }
        _ => {}
    }
}


fn prompt(message: &str) -> String {
    print!("{}", message);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}

fn prompt_password(message: &str) -> String {
    print!("{}", message);
    io::stdout().flush().unwrap();
    let password = read_password().expect("Failed to read password");
    password.trim().to_string()
}