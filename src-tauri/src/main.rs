// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api_interface;

use std::fs;
use std::path;
use std::env;
use std::sync::Mutex;
use sled;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str, state: tauri::State<'_, Mutex<sled::Db>>) -> String {

    format!("Hello, {}! You current path is", name)
}

fn main() {
    let db = sled::open("./data.sled")
        .expect("Error opening local data");
    let db = Mutex::new(db);


    tauri::Builder::default()
        .manage(db)
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
