use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use napi::Error;
use serde_json::Value;
use crate::models::ServerStorage::{ServerRegistry, MinecraftServer};

pub fn add_server(new_server: &MinecraftServer, path: PathBuf) -> napi::Result<()> {
    // 1. Load existing servers as OWNED objects
    let mut servers: Vec<MinecraftServer> = if path.exists() {
        let content = fs::read_to_string(&path)
            .map_err(|_| Error::from_reason("Failed to read DB"))?;

        // Deserialize into actual data, not references
        serde_json::from_str(&content).unwrap_or_else(|_| vec![])
    } else {
        vec![]
    };

    // 2. Clone the reference to turn it into an owned object for the Vec
    servers.push(new_server.clone());

    // 3. Write the full list back
    let json = serde_json::to_string_pretty(&servers)
        .map_err(|_| Error::from_reason("Failed to serialize DB"))?;

    fs::write(path, json)
        .map_err(|_| Error::from_reason("Failed to write DB file"))?;

    Ok(())
}

