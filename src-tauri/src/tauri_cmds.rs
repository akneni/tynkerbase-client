use tauri::{self, State};
use tokio::runtime::Runtime;
use tynkerbase_universal::netwk_utils::{self, Node, NodeDiags};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use anyhow::{anyhow, Result};

use crate::global_state::GlobalState;
use crate::agent_interface;

#[tauri::command]
pub fn list_nodes(state: State<Arc<Mutex<GlobalState>>>) -> Vec<HashMap<String, String>> {
    let rt = Runtime::new().unwrap();

    let lock = state.lock().unwrap();
    let mut nodes = vec![];
    for n in lock.nodes.iter() {
        nodes.push(n.to_hashmap());
    }

    
    let f = agent_interface::check_node_states(&lock);
    let res = rt.block_on(f);
    drop(lock);

    for n in nodes.iter_mut() {
        let id = n.get("node_id").unwrap();
        let status = if *res.get(id).unwrap_or(&false) {
            "active".to_string()
        }
        else {
            "inactive".to_string()
        };
        n.insert("status".to_string(), status);
    }

    println!("DEBUG: {:?}", nodes);

    nodes
}

#[tauri::command]
pub fn get_diags(node_id: &str, state: State<Arc<Mutex<GlobalState>>>) -> NodeDiags {
    let rt = Runtime::new().unwrap();

    let lock = state.lock().unwrap();
    let tyb_key = lock.tyb_key.clone();
    let mut endpoint = String::new();
    let mut name = String::new();
    for n in lock.nodes.iter() {
        if n.node_id == node_id {
            endpoint = n.addr.clone();
            name = n.name.clone();
            break;
        }
    }
    drop(lock);

    let f = agent_interface::get_diags(&endpoint, &tyb_key);
    rt.block_on(f).unwrap_or(NodeDiags::new(node_id, &name))
}