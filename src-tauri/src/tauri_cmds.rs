use tauri::{self, State, InvokeError};
use tokio::runtime::Runtime;
use tynkerbase_universal::netwk_utils::{self, Node, NodeDiags};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::global_state::GlobalState;
use crate::agent_interface;


#[tauri::command]
pub fn ping(node_id: &str, state: State<Arc<Mutex<GlobalState>>>) -> bool {
    let lock = state.lock().unwrap();
    let tyb_key = lock.tyb_key.clone();
    let mut node = None;
    for n in lock.nodes.iter() {
        if n.node_id == node_id {
            node = Some(n.clone());
            break;
        }
    }
    drop(lock);

    if let Some(node) = node {
        let rt = Runtime::new().unwrap();
        let f = agent_interface::ping(node.addr);
        return rt.block_on(f).is_ok();
    }

    false
}

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
pub async fn get_diags(node_id: &str, state: State<'_, Arc<Mutex<GlobalState>>>) -> Result<NodeDiags, InvokeError> {
    let (node, tyb_key) = query_node(node_id, &state);

    let node = match node {
        Some(n) => n,
        None => return Err(InvokeError::from("node_id does not exist")),
    };

    let f = agent_interface::get_diags(&node.addr, &tyb_key); // makes an API call
    let diags = f.await.unwrap_or(NodeDiags::new(node_id, &node.name));
    #[cfg(debug_assertions)] {
        println!("\n\ntauri command [fn get_diags] called. Result:\n{:#?}\n\n", diags);
    }
    Ok(diags)
}

#[tauri::command]
pub fn get_container_stats(node_id: &str, state: State<Arc<Mutex<GlobalState>>>) -> Vec<HashMap<String, String>> {
    let lock = state.lock().unwrap();
    let tyb_key = lock.tyb_key.clone();
    let mut node = None;
    for n in lock.nodes.iter() {
        if n.node_id == node_id {
            node = Some(n.clone());
            break;
        }
    }
    drop(lock);

    if let Some(node) = node {
        let rt = Runtime::new().unwrap();

        let f = agent_interface::list_container_stats_all(&node.addr, &tyb_key);
        let res = rt.block_on(f);
        if let Ok(res) = res {
            return res;
        }
        else if let Err(e) = res {
            #[cfg(debug_assertions)] println!("function agent_interface::list_container_stats_all return error [fn get_container_stats]: {}", e);
        }
    }
    #[cfg(debug_assertions)] println!("Error, no node matched that id");
    vec![]
}

fn query_node(node_id: &str, state: &State<Arc<Mutex<GlobalState>>>) -> (Option<Node>, String) {
    let lock = state.lock().unwrap();
    let tyb_key = lock.tyb_key.clone();
    for n in lock.nodes.iter() {
        if n.node_id == node_id {
            return (Some(n.clone()), tyb_key);
        }
    }
    (None, tyb_key)
}