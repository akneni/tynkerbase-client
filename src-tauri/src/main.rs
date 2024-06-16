// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api_interface;

use tynkerbase_universal::{
    crypt_utils::{
        self, compression_utils, rsa_utils, BinaryPacket, CompressionType, RsaKeys
    }, 
    docker_utils, 
    proj_utils::{self, FileCollection}
};

use std::fs;
use std::path;
use std::env;
use std::sync::Mutex;
use bincode;
use sled;
use tokio;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str, state: tauri::State<'_, Mutex<sled::Db>>) -> String {

    format!("Hello, {}! You current path is", name)
}

async fn test() {
    let res = api_interface::get_pub_key("http://192.168.101.241:7462")
        .await;

    let res = match res {
        Ok(r) => r,
        Err(e) => {
            println!("Error: {e}");
            std::process::exit(0);
        }
    };

    let apikey = "tyb_key_xuJHXGzPGsK7AcdxAYJRdZQtWvDsBSdksR8lFDPFTMAxmMuTyeCuoJM8L1e2dDyt";

    let mut packet = BinaryPacket::new();
    packet.attach_key(&apikey).unwrap();

    let packet = bincode::serialize(&packet).unwrap();

    let res = api_interface::create_proj("http://192.168.101.241:7462", "test-proj-1", packet)
        .await
        .unwrap();


    // let ig: Vec<String> = vec![];
    // let files = FileCollection::load("/home/aknen/Documents/TCP/workspace-1/", &ig)
    //     .unwrap();

    // let mut packet = BinaryPacket::from(&files)
    //     .unwrap();
    
    // packet.attach_key(&apikey)
    //     .unwrap();
    
    // rsa_utils::encrypt(&mut packet, &pubkey)
    //     .unwrap();

    // let packet = bincode::serialize(&packet)
    //     .unwrap();


}

fn main() {
    let f = test();
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(f);
    return;



    let db = sled::open("./data.sled")
        .expect("Error opening local data");
    let db = Mutex::new(db);


    tauri::Builder::default()
        .manage(db)
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
