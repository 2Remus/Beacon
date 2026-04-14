use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use napi::Error;
use serde_json::Value;
use crate::models::ServerStorage::{ServerRegistry, MinecraftServer};

pub fn add_server(new_server: &MinecraftServer, path: PathBuf) -> napi::Result<()> {
    // 1. Read and Close immediately
    let mut servers: Vec<MinecraftServer> = {
        if path.exists() {
            let content = fs::read_to_string(&path)
                .map_err(|e| Error::from_reason(format!("Read conflict: {}", e)))?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            vec![]
        }
    }; // 'content' and file handle are dropped here

    // 2. Modify data
    servers.push(new_server.clone());
    let json = serde_json::to_string_pretty(&servers)
        .map_err(|_| Error::from_reason("Serialization failed"))?;

    // 3. Atomic Write (The Windows Hang-Fixer)
    let temp_path = path.with_extension("tmp");

    // Write to a temp file first
    fs::write(&temp_path, json)
        .map_err(|e| Error::from_reason(format!("Temp write blocked: {}", e)))?;

    // Rename is atomic on Windows and will fail fast instead of hanging
    // if the original file is still locked.
    fs::rename(&temp_path, &path)
        .map_err(|e| Error::from_reason(format!("File is busy/locked: {}", e)))?;

    Ok(())
}

