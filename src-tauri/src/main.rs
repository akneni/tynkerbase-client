// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


#![allow(unused)] // TEMPORARY, REMOVE BEFORE PROD  

mod api_interface;
mod api_auth_interface;
mod consts;
mod global_state;
mod config;

use tynkerbase_universal::{
    crypt_utils::{
        self, 
        compression_utils, 
        hash_utils, 
        BinaryPacket, 
        CompressionType 
    }, 
    docker_utils, 
    proj_utils::{self, FileCollection}
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


use consts::{ACCT_INFO, STATE_PATH};
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


async fn check_node_states(state: &GlobalState) -> Vec<(String, bool)> {
    let mut res = vec![];

    let mut futures = vec![];
    for n in state.nodes.iter() {
        let endpoint = format!("https://{}:7462", n.ip_addr.as_ref().unwrap());
        let f = api_interface::ping(endpoint);
        let handle = tokio::spawn(f);
        futures.push((n.name.clone(), handle));
    }

    for (n, h) in futures {
        res.push((n, h.await.is_ok()));
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
    AttachNode {
        #[arg(short, long)]
        name: String,
        ip_addr: String,
    },
    SetUpstream {
        #[arg(short, long, default_value_t = String::new())]
        name: String
    }
}


fn login() -> GlobalState {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let email = prompt("Enter your email: ");
    let pass = prompt_password("Enter your password: ");

    let key = api_auth_interface::login(&email, &pass);
    let key = match rt.block_on(key) {
            Ok(r) => r,
            Err(e) => {
                println!("Error logging in: {e}");
                let _ = fs::remove_file(STATE_PATH);
                std::process::exit(1);
            }
    };
    let mut gstate = GlobalState::default();
    gstate.email = Some(email);
    gstate.pass_sha384 = Some(hash_utils::sha384(&pass));
    gstate.tyb_key = Some(key);
    gstate.save(STATE_PATH)
        .unwrap();
    gstate
}

fn main() {
    // Parse CLI commands
    let cmds = Cli::parse();

    // Create tokio runtime
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Load Global State
    let state_path = Path::new(STATE_PATH);
    let mut gstate = if !state_path.exists() && cmds.command != TopLevelCmds::Login {
        println!("You must login first.");
        let r = login();
        println!("Log in successful!\n");
        r
    }
    else {
        let mut gstate = GlobalState::load(STATE_PATH)
            .expect("Failed to load global state");
        if gstate.email.is_none() {
            println!("You must login first.");
            let r = login();
            println!("Log in successful!\n");
            r
        }
        else {
            gstate
        }
    };

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
                    if n == &nl.name {
                        endpoints.push(nl.ip_addr.clone().unwrap());
                        continue 'loop1;
                    }
                }
                println!("WARNING: no upstream node found for {}", &n);
            }

            println!("Transferring files...");
            let mut handles = vec![];
            for e in endpoints.iter() {
                let e = format!("https://{}:7462", e);
                let f = api_interface::transfer_files(e, &conf.proj_name, "./", gstate.tyb_key.as_ref().unwrap());
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
                let status = if *status_map.get(&n.name).unwrap_or(&false) {
                    "active"
                }
                else {
                    "inactive"
                };
                table.add_row(row![&n.name, n.ip_addr.as_ref().unwrap(), status]);
            }
            table.printstd();
        }
        TopLevelCmds::AttachNode { mut name, ip_addr } => {
            for n in gstate.nodes.iter() {
                if n.name == name || name.len() == 0 {
                    println!("Node names cannot be empty and must be unique. Please try again.");
                    process::exit(0);
                }
            }

            println!("Attaching node ...");
            let endpoint = format!("https://{}:7462", &ip_addr);
            let f = api_interface::get_id(&endpoint, gstate.tyb_key.as_ref().unwrap());
            let id = rt.block_on(f);
            let id = match id {
                Ok(r) => r,
                Err(e) => {
                    println!("Unable to reach `{}` -> {}", &ip_addr, e);
                    process::exit(1);
                }
            };

            let new_node = global_state::Node {
                name: name,
                id: id,
                ip_addr: Some(ip_addr),
            };

            gstate.nodes.push(new_node);
            gstate.save(STATE_PATH)
                .unwrap();
            println!("Added upstream node!");

        }
        TopLevelCmds::SetUpstream { mut name } => {
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