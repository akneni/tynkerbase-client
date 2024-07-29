use std::{
    process, 
    time::Duration,
    collections::HashMap,
};

use reqwest::{self, ClientBuilder};
use anyhow::{anyhow, Result};
use serde::{Serialize, Deserialize};

use crate::consts::NG_SKIP_WARN;
use crate::global_state::GlobalState;
use tynkerbase_universal::{
    constants::TYB_APIKEY_HTTP_HEADER, 
    crypt_utils::{compression_utils, BinaryPacket}, 
    file_utils, 
    netwk_utils::{NodeDiags, ProjConfig},
};

/// If this function returns an Ok(()) value, the node is up and running.
pub async fn ping(endpoint: String) -> Result<()> {
    let endpoint = parse_endpoint(endpoint)?;
    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true) // Disable TLS certificate validation
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    let res = client
        .get(format!("{}", endpoint))
        .header(NG_SKIP_WARN, "easter egg here")
        .send()
        .await
        .map_err(|e| anyhow!("Error sending https request: {e}"))?;

    validate_response(res).await?;
    Ok(())
}

pub async fn get_id(endpoint: &str, tyb_key: &str) -> Result<String> {
    let endpoint = parse_endpoint(endpoint)?;
    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true) 
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    let res = client
        .get(format!("{}", endpoint))
        .header(TYB_APIKEY_HTTP_HEADER, tyb_key)
        .header(NG_SKIP_WARN, "easter egg here")
        .send()
        .await
        .map_err(|e| anyhow!("Error sending https request: {e}"))?;

    let res = validate_response(res).await?;
    let id = res.text().await
        .map_err(|e| anyhow!("Error extracting text from response -> {}", e))?;

    Ok(id)
}

pub async fn create_proj(endpoint: &str, name: &str, tyb_key: &str) -> Result<()> {
    let endpoint = parse_endpoint(endpoint)?;
    let endpoint = format!("{}/files/proj/create-proj?name={}&confirm=false", endpoint, name);

    let res = reqwest::ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(5))
        .build()? 
        .get(endpoint)
        .header(TYB_APIKEY_HTTP_HEADER, tyb_key)
        .header(NG_SKIP_WARN, "easter egg here")
        .send()
        .await?;
    
    validate_response(res).await?;
    Ok(())
}

pub async fn delete_proj(endpoint: &str, name: &str, tyb_key: &str) -> Result<()> {
    let endpoint = parse_endpoint(endpoint)?;
    let endpoint = format!("{}/files/proj/delete-proj?name={}&confirm=false", endpoint, name);

    let res = reqwest::ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(5))
        .build()? 
        .get(endpoint)
        .header(TYB_APIKEY_HTTP_HEADER, tyb_key)
        .header(NG_SKIP_WARN, "easter egg here")
        .send()
        .await?;
    
    validate_response(res).await?;
    Ok(())
}

pub async fn transfer_files(endpoint: &str, name: &str, tyb_key: &str, files: &file_utils::FileCollection) -> Result<()> {
    let endpoint = parse_endpoint(endpoint)?;

    let mut packet = BinaryPacket::from(files)?;
    // compression_utils::compress_brotli(&mut packet)?;

    let packet_bin = bincode::serialize(&packet)
        .map_err(|e| anyhow!("Error serializing binary packet: {}", e))?;

    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true) // Disable TLS certificate validation
        .timeout(Duration::from_secs(10))
        .build()?;

    let res = client
        .post(format!("{}/files/proj/add-files-to-proj?name={}", endpoint, name))
        .body(packet_bin)
        .header(TYB_APIKEY_HTTP_HEADER, tyb_key)
        .header(NG_SKIP_WARN, "easter egg here")
        .send()
        .await
        .map_err(|e| anyhow!("Error sending https request: {e}"))?;

    validate_response(res).await?;
    Ok(())
}

pub async fn deploy_proj(endpoint: &str, name: &str, tyb_key: &str, files: &file_utils::FileCollection) -> Result<()> {
    purge_project(endpoint, name, tyb_key).await?;
    create_proj(endpoint, name, tyb_key).await?;
    transfer_files(endpoint, name, tyb_key, files).await?;
    Ok(())
}

pub async fn list_projects(endpoint: &str, tyb_key: &str) -> Result<Vec<String>> {
    let endpoint = parse_endpoint(endpoint)?;

    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true) 
        .timeout(Duration::from_secs(5))
        .build()?;

    let res = client
        .get(format!("{}/files/proj/list-projects", endpoint))
        .header(TYB_APIKEY_HTTP_HEADER, tyb_key)
        .header(NG_SKIP_WARN, "easter egg here")
        .send()
        .await
        .map_err(|e| anyhow!("Error sending https request: {e}"))?;

    let res = validate_response(res).await?;
    let body = res.bytes().await
        .map_err(|e| anyhow!("Error extracting bytes from response -> {}", e))?;
    
    let projects: Vec<String> = bincode::deserialize(&body)
        .map_err(|e| anyhow!("Error deserializing data -> {}", e))?;

    Ok(projects)
}

/// Returns a vector of hashmaps of all the containers running on the machine.
/// Each of the hashmaps have the following keys: `container_id`   `image`     `command`   `created_at`   `status`    `ports`     `names`
pub async fn list_containers(endpoint: &str, tyb_key: &str) -> Result<Vec<HashMap<String, String>>> {
    let endpoint = parse_endpoint(endpoint)?;

    let name_mappings: HashMap<&str, &str> = vec![
        ("CONTAINER ID", "container_id"),
        ("IMAGE", "image"),
        ("COMMAND", "command"),
        ("CREATED AT", "created_at"),
        ("STATUS", "status"),
        ("PORTS", "ports"),
        ("NAMES", "names"),
    ].into_iter().collect();

    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true) 
        .timeout(Duration::from_secs(5))
        .build()?;

    let res = client
        .get(format!("{}/docker/proj/list-containers", endpoint))
        .header(TYB_APIKEY_HTTP_HEADER, tyb_key)
        .header(NG_SKIP_WARN, "easter egg here")
        .send()
        .await
        .map_err(|e| anyhow!("Error sending https request: {e}"))?;

    let res = validate_response(res).await?;

    let text = res
        .text()
        .await
        .map_err(|e| anyhow!("Error extracting text from response [fn list_containers] => {}", e))?;

    let table = text
        .split("\n")
        .filter(|&s| s.trim().len() != 0)
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    // Change keys 
    let mut table = cvt_hashmap(table, "|||");
    for t in table.iter_mut() {
        for (&old_k, &new_k) in name_mappings.iter() {
            if let Some(value) = t.remove(old_k) {
                t.insert(new_k.to_string(), value);
            }
            else {
                #[cfg(debug_assertions)] {
                    println!("\n\nERROR FORMATTING RES: {:?}\n\n", t)
                }
            }
        }
    }

    Ok(table)
}


/// Returns a vector of hashmaps. Each hashmap has the following keys
/// `container_id` `container`   `cpu_perc`     `mem_usage_limit`   `mem_perc`     `net_io`   `block_io`   `pids`
pub async fn list_container_stats(endpoint: &str, tyb_key: &str) -> Result<Vec<HashMap<String, String>>> {
    let endpoint = parse_endpoint(endpoint)?;

    let name_mappings: HashMap<&str, &str> = vec![
        ("CONTAINER ID", "container_id"),
        ("CONTAINER", "container"),
        ("CPU %", "cpu_perc"),
        ("MEM USAGE / LIMIT", "mem_usage_limit"),
        ("MEM %", "mem_perc"),
        ("NET I/O", "net_io"),
        ("BLOCK I/O", "block_io"),
        ("PIDS", "pids")
    ].into_iter().collect();

    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true) 
        .timeout(Duration::from_secs(5))
        .build()?;

    let res = client
        .get(format!("{}/docker/proj/list-container-stats", endpoint))
        .header(TYB_APIKEY_HTTP_HEADER, tyb_key)
        .header(NG_SKIP_WARN, "easter egg here")
        .send()
        .await
        .map_err(|e| anyhow!("Error sending https request: {e}"))?;

    let res = validate_response(res).await?;
    let text = res.text().await
        .map_err(|e| anyhow!("Error extracting bytes from response -> {}", e))?
        .split("\n")
        .filter(|&s| s.trim().len() != 0)
        .map(|s| s.to_string())
        .collect::<Vec<String>>();    

    // Change keys 
    let mut table = cvt_hashmap(text, "|||");
    for t in table.iter_mut() {
        for (&old_k, &new_k) in name_mappings.iter() {
            if let Some(value) = t.remove(old_k) {
                t.insert(new_k.to_string(), value);
            }
        }
    }

    Ok(table)
}

/// Returns a vector of hashmaps. Each hashmap has the following keys
/// `container_id` `container`   `cpu_perc`     `mem_usage_limit`   `mem_perc`     `net_io`   `block_io`   `pids`
/// `image`     `command`   `created_at`   `status`    `ports`     `names`
pub async fn list_container_stats_all(endpoint: impl AsRef<str>, tyb_key: impl AsRef<str>) -> Result<Vec<HashMap<String, String>>> {
    let endpoint = endpoint.as_ref();
    let tyb_key = tyb_key.as_ref();
    let (lst, stats) = tokio::join!(
        list_containers(endpoint, tyb_key),
        list_container_stats(endpoint, tyb_key),
    );

    let mut result = lst?;
    if let Ok(stats) = stats {
        for stat in stats {
            let s_id = stat.get("container_id").unwrap();
            for r in result.iter_mut() {
                let r_id = r.get("container_id").unwrap();
                if s_id == r_id {
                    r.extend(stat);
                    break;
                }
            }
        }
    }

    // Parse the `ports` field to make it more readable
    for cont in result.iter_mut() {
        let p = match cont.get("ports") {
            Some(p) => p,
            None => break,
        };
        cont.insert("ports".to_string(), parse_port_str(p));
    }
    

    Ok(result)
}

pub async fn build_img (endpoint: &str, name: &str, tyb_key: &str) -> Result<()> {
    let endpoint = parse_endpoint(endpoint)?;

    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true) 
        .timeout(Duration::from_secs(2000))
        .build()?;

    let res = client
        .get(format!("{}/docker/proj/build-img?name={}", endpoint, name))
        .header(TYB_APIKEY_HTTP_HEADER, tyb_key)
        .header(NG_SKIP_WARN, "easter egg here")
        .send()
        .await
        .map_err(|e| anyhow!("Error sending https request [fn build_img]: {e}"))?;

    validate_response(res).await?;
    Ok(())
}

pub async fn spawn_container(endpoint: &str, config: &ProjConfig, tyb_key: &str) -> Result<()> {
    let endpoint = parse_endpoint(endpoint)?;

    let data = bincode::serialize(config)
        .map_err(|e| anyhow!("Failed to serialize config [fn spawn_container] -> {}", e))?;

    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true) 
        .timeout(Duration::from_secs(12))
        .build()?;

    let res = client
        .post(format!("{}/docker/proj/spawn-container", endpoint))
        .header(TYB_APIKEY_HTTP_HEADER, tyb_key)
        .header(NG_SKIP_WARN, "easter egg here")
        .body(data)
        .send()
        .await
        .map_err(|e| anyhow!("Error sending https request [fn spawn_container]: {e}"))?;

    validate_response(res).await?;
    Ok(())
}

pub async fn purge_project(endpoint: &str, name: &str, tyb_key: &str) -> Result<()> {
    let endpoint = parse_endpoint(endpoint)?;

    #[cfg(debug_assertions)] println!("ENDPOINT: {}", endpoint);

    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true) 
        .timeout(Duration::from_secs(12))
        .build()?;

    let res = client
        .get(format!("{}/files/proj/purge-project?name={}", endpoint, name))
        .header(TYB_APIKEY_HTTP_HEADER, tyb_key)
        .header(NG_SKIP_WARN, "easter egg here")
        .send()
        .await
        .map_err(|e| anyhow!("Error sending https request [fn purge_project]: {e}"))?;

    #[cfg(debug_assertions)] println!("request sent successfully");


    validate_response(res).await?;

    #[cfg(debug_assertions)] println!("purge project validated");

    Ok(())
}

pub async fn check_node_states(state: &GlobalState) -> HashMap<String, bool> {
    let mut res = HashMap::new();

    let mut futures = vec![];
    for n in state.nodes.iter() {
        let f = ping(n.addr.clone());
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

pub async fn get_diags(endpoint: &str, tyb_key: &str) -> Result<NodeDiags> {
    let endpoint = parse_endpoint(endpoint)?;

    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true) 
        .timeout(Duration::from_secs(7))
        .build()?;

    let res = client
        .get(format!("{}/diags/get-diags", &endpoint))
        .header(TYB_APIKEY_HTTP_HEADER, tyb_key)
        .header(NG_SKIP_WARN, "easter egg here")
        .send()
        .await
        .map_err(|e| anyhow!("Error sending https request [fn get_diags]: {e}"))?;

    let text = res
        .text()
        .await
        .map_err(|e| anyhow!("Error extracting text from https response [fn get_diags]: {e}"))?;

    let diags: NodeDiags = serde_json::from_str(&text)
        .map_err(|e| anyhow!("Error deserializing json response from agent [fn get_diags]: {e}"))?;

    Ok(diags)
}


fn parse_endpoint(endpoint: impl Into<String>) -> Result<String> {
    let endpoint: String = endpoint.into();
    if endpoint.starts_with("http") {
        return Ok(endpoint);
    }

    let is_ip = endpoint.len() < 16 && endpoint
        .chars()
        .all(|c| ".0123456789".contains(c));

    if is_ip {
        return Ok(format!("https://{}:7462", endpoint))
    }

    Err(anyhow!("Endpoint `{}` is in an unexpected format.", endpoint))
}

pub async fn validate_response(response: reqwest::Response) -> Result<reqwest::Response> {
    if !response.status().is_success() {
        let status = response.status();
        let mut text = response.text().await.unwrap_or("NONE".to_string());
        return Err(anyhow!("\nNon 200 response from node\nStatus Code: {:?}\nText Body: {}\n", status, text));
    }
    Ok(response)
}

/// Parses the default port string returned by docker to XXXX/tcp where XXXX is the host's exposed port
fn parse_port_str(port: &str) -> String {
    let port = match port.split_once(":::") {
        Some(p) => p,
        None => return port.to_string(),
    }.1;

    let host_p = match port.split_once("->") {
        Some(p) => p,
        None => return port.to_string(),
    }.0;
    
    let conn_type = match port.split_once("/") {
        Some(p) => p,
        None => return port.to_string(),
    }.1;

    format!("{}/{}", host_p, conn_type)
}

/// Parses the response of a table 
fn cvt_hashmap(v: Vec<String>, split_char: &str) -> Vec<HashMap<String, String>> {
    let mut map = vec![];

    let headers = v[0]
        .clone()
        .split(split_char)
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    for row in v.into_iter().skip(1) {
        let row = row
            .split(split_char)
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        let mut ph = HashMap::new();
        for (h, v) in headers.iter().zip(row.iter()) {
            ph.insert(h.clone(), v.clone());
        }
        map.push(ph);
    }

    map
}