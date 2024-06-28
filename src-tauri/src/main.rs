// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


#![allow(unused)] // TEMPORARY, REMOVE BEFORE PROD  

mod api_interface;
mod api_auth_interface;
mod constants;
mod global_state;
mod config;

use tynkerbase_universal::{
    crypt_utils::{
        self, compression_utils, hash_utils, BinaryPacket, CompressionType
    }, 
    docker_utils, 
    proj_utils::{self, FileCollection}
};

use std::{
    fs,
    path::Path, 
    process::exit,
};
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
    // #[command(alias="--help")]
    // Help,
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
            println!("A graphical app is coming soon!");
            return;
            // launch_gui(gstate);
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
            gstate.save(ACCT_INFO)
                .unwrap();
        },
        TopLevelCmds::CreateAccount { email, password } => {
            let f = api_auth_interface::create_account(&email, &password);
            let res = rt.block_on(f);
            res.unwrap();
        }
        TopLevelCmds::Deploy => {
            let conf = fs::read_to_string(".tynkerbase-config.json").unwrap();
            let conf: config::Config = serde_json::from_str(&conf).unwrap();

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

            let mut handles = vec![];
            for e in endpoints.iter() {
                let f = api_interface::transfer_files(e, &conf.proj_name, "./", gstate.tyb_key.as_ref().unwrap());
                handles.push(f);
            }

            for h in handles {
                let res = rt.block_on(h);
                if let Err(e) = res {
                    println!("Warning: error pushing changes node {e}");
                }
            }
        }
        TopLevelCmds::Init {name} => {
            let conf_path = Path::new(".tynkerbase-config.json");
            if conf_path.exists() {
                println!("Current directory is already a project!");
                exit(1);
            }

            let mut conf = config::Config::default();
            conf.proj_name = name;
            let conf = serde_json::to_string_pretty(&conf)
                .expect("If you're seeing this error, send out a bug report.");

            fs::write(".tynkerbase-config.json", &conf)
                .unwrap();
        },
        TopLevelCmds::ListNodes => {
            let mut gstate = gstate.clone();

            let section_width = 30;

            gstate.nodes.push(global_state::Node { name: "oh long johnson".to_string(), ip_addr: None });

            let mega_divider: String = std::iter::repeat('=').take(2*section_width+1).collect();
            let divider: String = std::iter::repeat('-').take(2*section_width+1).collect();

            println!("|{}|", mega_divider);
            println!("|{:^section_width$}|{:^section_width$}|\n|{}|", "Node Name", "IPv4 Address", mega_divider);

            for n in gstate.nodes.iter() {
                println!("|{:^section_width$}|{:^section_width$}|\n|{}|", &n.name, n.ip_addr.as_ref().unwrap_or(&"Unknown".to_string()), divider);
            }
        }
        TopLevelCmds::AttachNode { name, ip_addr } => {
            let mut node = global_state::Node::default();
            node.name = name;
            node.ip_addr = Some(ip_addr.clone());

            let endpoint = format!("https://{}", &ip_addr);
            let f = api_interface::ping(&endpoint);
            let res = rt.block_on(f);
            match res {
                Ok(_) => {
                    println!("Added upstream node!");
                    gstate.nodes.push(node);
                    gstate.save(ACCT_INFO);
                },
                Err(e) => println!("Failed to add upstream node: {}", e),
            }


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