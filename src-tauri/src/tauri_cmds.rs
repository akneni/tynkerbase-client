use tauri::{self, State, InvokeError};
use tynkerbase_universal::crypt_utils;
use tynkerbase_universal::netwk_utils::{self, Node, NodeDiags};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex as TkMutex;
use crate::global_state::GlobalState;
use crate::agent_interface;
use crate::api_auth_interface;


#[tauri::command]
pub async fn ping(node_id: &str, state: State<'_, Arc<TkMutex<GlobalState>>>) -> Result<bool, InvokeError> {
    let lock = state.lock().await;
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
        let f = agent_interface::ping(node.addr);
        return Ok(f.await.is_ok());
    }

    Ok(false)
}

#[tauri::command]
pub async fn login_account(email: &str, password: &str, state: State<'_, Arc<TkMutex<GlobalState>>>) -> Result<(), InvokeError> {
    let res = api_auth_interface::login(email, password).await;
    if let Err(e) = res {
        return Err(InvokeError::from(format!("Error -> {}", e)));
    }

    Ok(())
}

#[tauri::command]
pub async fn create_account(email: &str, password: &str, state: State<'_, Arc<TkMutex<GlobalState>>>) -> Result<(), InvokeError> {
    let res = api_auth_interface::create_account(email, password).await;
    if let Err(e) = res {
        return Err(InvokeError::from(format!("Error -> {}", e)));
    }

    Ok(())
}

#[tauri::command]
pub async fn delete_account(email: &str, password: &str, state: State<'_, Arc<TkMutex<GlobalState>>>) -> Result<(), InvokeError> {
    let res = api_auth_interface::delete_account(email, password).await;
    if let Err(e) = res {
        return Err(InvokeError::from(format!("Error -> {}", e)));
    }

    Ok(())
}

#[tauri::command]
pub async fn list_nodes(state: State<'_, Arc<TkMutex<GlobalState>>>) -> Result<Vec<HashMap<String, String>>, InvokeError> {
    let mut nodes = {
        let lock = state.lock().await;
        lock.nodes.iter().map(|n| n.to_hashmap()).collect::<Vec<_>>()
    };

    let res = {
        let lock = state.lock().await;
        agent_interface::check_node_states(&lock).await
    };

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

    Ok(nodes)
}

#[tauri::command]
pub async fn delete_node(node_id: &str, state: State<'_, Arc<TkMutex<GlobalState>>>) -> Result<(), InvokeError> {
    let (node, _) = query_node(node_id, &state).await;
    let node = match node {
        Some(n) => n,
        None => return Err(InvokeError::from("No node with that id.")),
    };  

    let (email, pass_sha256) = {
        let lock = state.lock().await;
        (lock.email.clone() ,crypt_utils::hash_utils::sha256(&lock.password))
    };

    let res = api_auth_interface::remove_node(&email, &pass_sha256, node_id).await;
    if let Err(e) = res {
        return Err(InvokeError::from(format!("Error calling API -> {}", e)));
    }

    Ok(())
}

#[tauri::command]
pub async fn get_diags(node_id: &str, state: State<'_, Arc<TkMutex<GlobalState>>>) -> Result<NodeDiags, InvokeError> {
    let (node, tyb_key) = query_node(node_id, &state).await;

    let node = match node {
        Some(n) => n,
        None => return Err(InvokeError::from("node_id does not exist")),
    };

    let f = agent_interface::get_diags(&node.addr, &tyb_key); // makes an API call
    let diags = f.await.unwrap_or(NodeDiags::new(node_id, &node.name));
    Ok(diags)
}

#[tauri::command]
pub async fn get_container_stats(node_id: &str, state: State<'_, Arc<TkMutex<GlobalState>>>) -> Result<Vec<HashMap<String, String>>, InvokeError> {
    let (node, tyb_key) = query_node(node_id, &state).await;

    if let Some(node) = node {
        let res = agent_interface::list_container_stats_all(node.addr.clone(), tyb_key.clone()).await;
        if let Ok(res) = res {
            #[cfg(debug_assertions)] println!("Successfully called `get_container_stats`. result -> {:#?}", res);
            return Ok(res);
        }
        else if let Err(e) = res {
            #[cfg(debug_assertions)] println!("function agent_interface::list_container_stats_all return error [fn get_container_stats]: {}", e);
        }
    }
    Err(InvokeError::from("No node with that node id"))
}

async fn query_node(node_id: &str, state: &State<'_, Arc<TkMutex<GlobalState>>>) -> (Option<Node>, String) {
    let lock = state.lock().await;
    let tyb_key = lock.tyb_key.clone();
    for n in lock.nodes.iter() {
        if n.node_id == node_id {
            return (Some(n.clone()), tyb_key);
        }
    }
    (None, tyb_key)
}