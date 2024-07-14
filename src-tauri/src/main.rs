// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


#![allow(unused)] // TEMPORARY, REMOVE BEFORE PROD  

mod api_interface;
mod api_auth_interface;
mod consts;
mod global_state;
mod config;

use tauri;
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
    file_utils::{self, FileCollection},
    netwk_utils::Node,
};
use consts::PROJ_JSON_CONFIG;
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

    let mut futures = vec![];
    for n in state.nodes.iter() {
        let f = api_interface::ping(n.addr.clone());
        let handle = tokio::spawn(f);
        futures.push((n.node_id.clone(), handle));
    }

    for (n, h) in futures {
        if let Ok(Ok(_)) = h.await {
            res.insert(n,  true);
        }
        else {
            res.insert(n,  false);
        }
    }

    res
}

fn prompt_node<'a>(gstate: &'a GlobalState) -> &'a Node {
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
    &gstate.nodes[idx]
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
        #[arg(long, default_value_t = String::new())]
        name: String
    },
    ListNodes,
    ListProjects {
        #[arg(long, default_value_t = String::new())]
        name: String,
    },
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
            let conf = fs::read_to_string(PROJ_JSON_CONFIG)
                .expect("Error, not a valid tynkerbase project");
            let conf: config::Config = serde_json::from_str(&conf).unwrap();

            if conf.node_names.len() == 0 {
                println!("No upstream nodes set. Use `tyb add-upstream` configure an upstream node");
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
                        endpoints.push(nl);
                        continue 'loop1;
                    }
                }
                println!("WARNING: no upstream node found for node name `{}`", &n);
                #[cfg(debug_assertions)] println!("\n\nNodes: \n{:#?}\n\n", gstate.nodes);
            }

            if endpoints.len() == 0 {
                println!("No valid upstream nodes found.");
                process::exit(0);
            }

            let mut failed_deployments = vec![];

            let files = file_utils::FileCollection::load("./", &conf.ignore)
                .unwrap();

            println!("Transferring files...\nPayload Size: {} MB", files.sizeof() as f64 / 1_000_000.);
            let mut handles = vec![];
            for &e in endpoints.iter() {
                let f = api_interface::deploy_proj(&e.addr, &conf.proj_name, &gstate.tyb_key, &files);
                handles.push((f, e));
            }

            for (handle, node) in handles {
                let res = rt.block_on(handle);
                if let Err(e) = res {
                    failed_deployments.push(format!("Failed to transfer files to node `{}` -> {}", &node.name, e));
                    endpoints.retain(|&e| e.name != node.name);
                }
            }

            println!("Building Images...");
            let mut handles = vec![];
            for &e in endpoints.iter() {
                let f = api_interface::build_img(&e.addr, &conf.proj_name, &gstate.tyb_key);
                handles.push((f, e));
            }

            for (handle, node) in handles {
                let res = rt.block_on(handle);
                if let Err(e) = res {
                    failed_deployments.push(format!("Failed to build image on node `{}` -> {}", node.name, e));
                    endpoints.retain(|&e| e.name != node.name);

                }
            }

            println!("Starting up containers...");
            let mut handles = vec![];
            for &e in endpoints.iter() {
                let f = api_interface::spawn_container(&e.addr, &conf.proj_name, &gstate.tyb_key);
                handles.push((f, e));
            }

            for (handle, node) in handles {
                let res = rt.block_on(handle);
                if let Err(e) = res {
                    failed_deployments.push(format!("Failed to spawn container on node `{}` -> {}", node.name, e));
                }
            }

            if failed_deployments.len() != 0 {
                println!("\n\n\nERROR SUMMARY:\n");
                for (i, msg) in failed_deployments.iter().enumerate() {
                    println!("{:2<})    {}\n\n", i, msg);
                }
            }

        }
        TopLevelCmds::Init { mut name } => {
            let conf_path = Path::new(PROJ_JSON_CONFIG);
            if conf_path.exists() {
                println!("Current directory is already a project!");
                process::exit(1);
            }

            if name.len() == 0 {
                name = crypt_utils::prompt("Please name this project: ");
            }

            let mut conf = config::Config::default();
            conf.proj_name = name;
            let conf = serde_json::to_string_pretty(&conf)
                .expect("If you're seeing this error, send out a bug report.");

            fs::write(PROJ_JSON_CONFIG, &conf)
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
        TopLevelCmds::ListProjects { mut name } => {
            let mut node = &gstate.nodes[0];
            if name.len() == 0 {
                node = prompt_node(&gstate); 
                name = node.name.clone();
                println!("\n\n");
            }
            else {
                let mut found_node = false;
                for n in gstate.nodes.iter() {
                    if n.name == name {
                        found_node = true;
                        node = n;
                    }
                }
                if !found_node {
                    println!("No node named `{}`", name);
                    process::exit(0);
                }
            }


            let f = api_interface::list_projects(&node.addr, &gstate.tyb_key);
            let res = rt.block_on(f);
            match res {
                Ok(v) =>  {
                    println!("PROJECTS:");
                    if v.len() == 0 {
                        println!("None");
                    }
                    for (i, p) in v.iter().enumerate() {
                        println!("{:2<})\t{}", i, p);
                    }
                }
                Err(e) => {
                    println!("Error fetching project data from node `{}` -> {:?}", name, e);
                }
            }
        }
        TopLevelCmds::AddUpstream { mut name } => {
            if name.len() == 0 {
                let node = prompt_node(&gstate);
                name = node.name.clone();
            }

            let conf = fs::read_to_string(PROJ_JSON_CONFIG)
                .expect("Error, not a valid tynkerbase project");
            let mut conf: config::Config = serde_json::from_str(&conf).unwrap();
            if conf.node_names.contains(&name) {
                println!("Node is already set as an upstream target.");
                process::exit(0);
            }
            conf.node_names.push(name);
            let conf = serde_json::to_string_pretty(&conf).unwrap();
            fs::write(PROJ_JSON_CONFIG, conf)
                .expect("Unable to write to config file.");

        }
        _ => {}
    }
}

