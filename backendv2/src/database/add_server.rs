use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use serde_json::Value;
use crate::models::ServerStorage::{ServerRegistry, MinecraftServer};

pub fn add_server(server: MinecraftServer, file_path: PathBuf) -> napi::Result<()> {
    let file = File::create(&file_path)
        .map_err(|e| napi::Error::from_reason("Failed to crate server file".to_string()))?;

    let buf_writer = BufWriter::new(&file);

    serde_json::to_writer_pretty(buf_writer, &server)
        .map_err(|e| println!("server add error: {}", e));

    Ok(())
}

