// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


#![allow(unused)] // TEMPORARY, REMOVE BEFORE PROD  

mod api_interface;
mod api_auth_interface;
mod consts;
mod global_state;
mod config;

use reqwest::header::ACCEPT;
use tynkerbase_universal::{
    crypt_utils::{
        self, 
        compression_utils, 
        hash_utils, 
        BinaryPacket, 
        CompressionType 
    }, 
    docker_utils, 
    proj_utils::{self, FileCollection},
    netwk_utils::Node,
};

use std::{
    fs::{self, remove_file},
    path::Path, 
    process,
};
use std::env;
use std::sync::{Mutex, Arc};
use std::collections::HashMap;
use std::io::{self, Write};
use rpassword::read_password;
use bincode;
use tokio::{self, runtime::Runtime};
use clap::{Parser, Subcommand};
use prettytable::{Table, Row, Cell, row};

use global_state::GlobalState;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str, state: tauri::State<'_, Mutex<GlobalState>>) -> String {
    format!("Hello, {}! You current path is", name)
}

fn launch_gui(state: GlobalState) {
    let state = Arc::new(Mutex::new(state));

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn check_node_states(state: &GlobalState) -> HashMap<String, bool> {
    let mut res = HashMap::new();

    #[cfg(debug_assertions)] println!("NODES -> {:?}", state.nodes);

    let mut futures = vec![];
    for n in state.nodes.iter() {
        let endpoint = format!("https://{}:7462", &n.addr);
        let f = api_interface::ping(endpoint);
        let handle = tokio::spawn(f);
        futures.push((n.node_id.clone(), handle));
    }

    for (n, h) in futures {
        res.insert(n,  h.await.is_ok());
    }

    res
}



#[derive(Parser)]
#[command(version = "0.0.1")]
struct Cli {
    #[command(subcommand)]
    command: TopLevelCmds,
}

#[derive(Subcommand, PartialEq, Eq)]
enum TopLevelCmds {
    Gui,
    Login,
    CreateAccount {
        #[arg(long, short)]
        email: String,
        #[arg(long, short)]
        password: String,
    },
    Deploy,
    Init {
        #[arg(short, long)]
        name: String
    },
    ListNodes,
    AddUpstream {
        #[arg(long, default_value_t = String::new())]
        name: String
    }
}


fn login() -> GlobalState {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let email = crypt_utils::prompt("Enter your email: ");
    let pass = crypt_utils::prompt_secret("Enter your password: ");

    let key = api_auth_interface::login(&email, &pass);
    let key = match rt.block_on(key) {
            Ok(r) => r,
            Err(e) => {
                println!("Error logging in: {e}");
                let _ = fs::remove_file(GlobalState::path());
                std::process::exit(1);
            }
    };
    let mut gstate = GlobalState::default();
    gstate.email = email;
    gstate.password = pass;
    gstate.tyb_key = key;
    gstate.save()
        .unwrap();
    gstate
}

fn main() {

    // Parse CLI commands
    let cmds = Cli::parse();

    // Create tokio runtime
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Load Global State
    let mut gstate = if GlobalState::exists() {
        GlobalState::load().unwrap()
    }
    else {
        let gstate = login();
        match cmds.command {
            TopLevelCmds::Login => process::exit(0),
            _ => {},
        }
        gstate
    };

    gstate.populate_nodes().unwrap();


    match cmds.command {
        TopLevelCmds::Gui => {
            launch_gui(gstate);
        },
        TopLevelCmds::Login => {
            login();
        },
        TopLevelCmds::CreateAccount { email, password } => {
            let f = api_auth_interface::create_account(&email, &password);
            let res = rt.block_on(f);
            res.unwrap();
        }
        TopLevelCmds::Deploy => {
            let conf = fs::read_to_string(".tynkerbase-config.json")
                .expect("Error, not a valid tynkerbase project");
            let conf: config::Config = serde_json::from_str(&conf).unwrap();

            if conf.node_names.len() == 0 {
                println!("No upstream nodes set. Use `tyb set-upstream` configure an upstream node");
                process::exit(0);
            }

            if !Path::new("Dockerfile").exists() {
                println!("Please create docker file before deploying");
                process::exit(0);
            }

            let mut endpoints = vec![];

            'loop1: for ref n in conf.node_names {
                for nl in gstate.nodes.iter() {
                    if n == &nl.node_id {
                        endpoints.push(nl.addr.clone());
                        continue 'loop1;
                    }
                }
                println!("WARNING: no upstream node found for {}", &n);
            }

            println!("Transferring files...");
            let mut handles = vec![];
            for e in endpoints.iter() {
                let e = format!("https://{}:7462", e);
                let f = api_interface::transfer_files(e, &conf.proj_name, "./", &gstate.tyb_key);
                handles.push(f);
            }

            #[cfg(debug_assertions)] {
                println!("Waiting on futures");
            }

            for h in handles {
                let res = rt.block_on(h);
                if let Err(e) = res {
                    println!("Warning: error pushing changes node -> {e}");
                }
            }
        }
        TopLevelCmds::Init {name} => {
            let conf_path = Path::new(".tynkerbase-config.json");
            if conf_path.exists() {
                println!("Current directory is already a project!");
                process::exit(1);
            }

            let mut conf = config::Config::default();
            conf.proj_name = name;
            let conf = serde_json::to_string_pretty(&conf)
                .expect("If you're seeing this error, send out a bug report.");

            fs::write(".tynkerbase-config.json", &conf)
                .unwrap();
        },
        TopLevelCmds::ListNodes => {
            let mut table = Table::new();

            table.set_titles(row!["Name", "Ip Addr", "Status"]);

            let status_map = rt.block_on(check_node_states(&gstate))
                .into_iter()
                .collect::<HashMap<String, bool>>();

            for n  in gstate.nodes.iter() {
                let status = if *(status_map.get(&n.node_id).unwrap_or(&false)){
                    "active"
                }
                else {
                    "inactive"
                };
                table.add_row(row![&n.name, &n.addr, status]);
            }
            table.printstd();
        }
        TopLevelCmds::AddUpstream { mut name } => {
            if name.len() == 0 {
                for (i, n) in gstate.nodes.iter().enumerate() {
                    println!("{})\t{}", i, &n.name);
                }
                println!("============================================");
                let idx = crypt_utils::prompt("Choose a node number: ");
                let idx: usize = match idx.parse() {
                    Ok(r) => r,
                    _ => {
                        println!("`{}` is not a valid number", idx);
                        process::exit(0);
                    }
                };
                if idx >= gstate.nodes.len() {
                    println!("`{}` is out of range.", idx);
                    process::exit(0);
                }
                name = gstate.nodes[idx].name.clone();
            }

            let conf = fs::read_to_string(".tynkerbase-config.json")
                .expect("Error, not a valid tynkerbase project");
            let mut conf: config::Config = serde_json::from_str(&conf).unwrap();
            if conf.node_names.contains(&name) {
                println!("Node is already set as an upstream target.");
                process::exit(0);
            }
            conf.node_names.push(name);
            let conf = serde_json::to_string_pretty(&conf).unwrap();
            fs::write(".tynkerbase-config.json", conf)
                .expect("Unable to write to config file.");

        }
        _ => {}
    }
}

