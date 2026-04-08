use std::{env, fs};
use std::fs::File;
use std::io::Cursor;
use std::path::PathBuf;
use std::process::Command;
use napi::Error;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use reqwest::Client;
use serde_json::Value;
use crate::containerizer::operations::create_container_env;
use crate::models::ServerStorage::MinecraftServer;

use crate::database::add_server::add_server;
use crate::models::container::ContainerConfig;
use crate::models::ServerStorage::Provider;
use tokio::fs::File as AsyncFile;
//use tokio::io::AsyncWriteExt;

#[napi]
pub async fn create_server(
    id: String,
    name: String,
    provider: Provider,
    version: String,
    ram_mb: u32,
    port: u32,
    online_mode: bool,
) -> napi::Result<String> {

    // 1. RESOLVE & DOWNLOAD (The Pull Step)
    // Pulls from cache or hits the API (Paper/Fabric/Vanilla)
    let jar_path = ensure_jar_exists(&provider, &version).await?;

    // 2. BUILD CONFIG (The Environment Metadata)
    // We initialize fully here to avoid the "partially assigned" E0381 error.
    let container_config = ContainerConfig {
        server_id: id.clone(), // Clone here so we can use 'id' again later
        jar_path: jar_path.to_string_lossy().into_owned(),
        port,
        ram_mb,
        enable_rcon: true,
        online_mode,
    };

    // 3. PROVISION (The Filesystem Layer)
    // Creates the /containers/id folder, symlinks the JAR, and writes server.properties
    let container_path = create_container_env(container_config)?;

    // 4. REGISTER (The Persistence Layer)
    let new_server = MinecraftServer {
        id: id.clone(),
        name,
        version,
        provider,
        port,
        ram: ram_mb,
        // Using the actual path returned by the provisioner
        instance_path: container_path.to_string_lossy().to_string(),
        status: "stopped".to_string(),
    };

    let db_path = crate::get_resource_path()?.join("db.json");
    add_server(&new_server, db_path)?;

    Ok(format!("Server {} ({}) provisioned at {:?}", id, &new_server.version, container_path))
}


#[napi]
pub async fn get_servers() -> napi::Result<Vec<MinecraftServer>> {
    let db_path = crate::get_resource_path()?.join("db.json");

    // We don't spawn a standard thread manually here;
    // we use tokio (which napi-rs uses under the hood for async)
    // to poll until the file is ready or just read it once.

    if !db_path.exists() {
        return Err(Error::from_reason("Database not found").into());
    }

    // Instead of a loop that disappears, we just perform the read.
    // If you need to wait for a specific condition, do it here.
        let content = std::fs::read_to_string(&db_path)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        let servers: Vec<MinecraftServer> = serde_json::from_str(&content)
            .map_err(|e| Error::from_reason(e.to_string()))?;

    Ok(servers)
}


async fn ensure_jar_exists(provider: &Provider, version: &str) -> napi::Result<PathBuf> {
    let cache_dir = crate::get_resource_path()?.join("cache");

    // Ensure the cache directory exists before we try to save files to it
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir).map_err(|e| napi::Error::from_reason(e.to_string()))?;
    }

    // Fixed: provider needs to be Displayed as a string for format!
    let jar_name = format!("{:?}-{}.jar", provider, version);
    let target_path = cache_dir.join(&jar_name);

    if target_path.exists() {
        return Ok(target_path);
    }

    // Create a client with a User-Agent (Required for Paper v3)
    let client = Client::builder()
        .user_agent("ProjectBeacon/1.0 (Software Development Student Project)")
        .build()
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    let download_url = match provider {
        Provider::Paper => {
            // 1. Get builds for the version
            let api_url = format!("https://api.papermc.io/v2/projects/paper/versions/{}", version);

            // Added explicit type annotation to the .json::<Value>() call
            let resp = client.get(&api_url).send().await
                .map_err(|e| napi::Error::from_reason(format!("Paper API failure: {}", e)))?
                .json::<Value>().await
                .map_err(|e| napi::Error::from_reason(format!("Invalid JSON from Paper: {}", e)))?;

            let build_id = resp["builds"].as_array()
                .and_then(|b| b.last())
                .and_then(|last| {
                    last.as_u64().map(|id| id.to_string())
                        .or_else(|| last.as_str().map(|s| s.to_string()))
                })
                .ok_or_else(|| napi::Error::from_reason(format!("No Paper builds found for version {}", version)))?;

            // 2. Get build info for the filename
            let build_url = format!("{}/builds/{}", api_url, build_id);

            // Added explicit type annotation here as well
            let build_info = client.get(&build_url).send().await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?
                .json::<Value>().await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;

            let file_name = build_info["downloads"]["application"]["name"].as_str()
                .ok_or_else(|| napi::Error::from_reason("Failed to resolve Paper filename"))?;

            // 3. Final URL
            format!("{}/downloads/{}", &build_url, file_name)
        },

        Provider::Vanilla => {
            let manifest_url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

            // Annotating the manifest fetch
            let manifest = client.get(manifest_url).send().await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?
                .json::<Value>().await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;

            let version_url = manifest["versions"].as_array()
                .and_then(|v| v.iter().find(|entry| entry["id"] == version))
                .and_then(|entry| entry["url"].as_str())
                .ok_or_else(|| napi::Error::from_reason(format!("Vanilla version {} not found", version)))?;

            // Annotating the metadata fetch
            let metadata = client.get(version_url).send().await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?
                .json::<Value>().await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;

            metadata["downloads"]["server"]["url"].as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| napi::Error::from_reason("No Vanilla server download link found"))?
        },

        Provider::Forge => {
            format!(
                "https://maven.minecraftforge.net/net/minecraftforge/forge/{0}/forge-{0}-installer.jar",
                version
            )
        },

        _ => return Err(napi::Error::from_reason("Unsupported provider selected")),
    };

    let response = client.get(&download_url).send().await
        .map_err(|e| napi::Error::from_reason(format!("Network error: {}", e)))?;

    // Check if the status is actually 200 OK
    if !response.status().is_success() {
        return Err(napi::Error::from_reason(format!("Server returned error: {}", response.status())));
    }

    let bytes = response.bytes().await
        .map_err(|e| napi::Error::from_reason(format!("Failed to read stream: {}", e)))?;

    // Fixed: Using tokio::fs::File (aliased as AsyncFile) and importing AsyncWriteExt for methods
    use tokio::io::AsyncWriteExt;
    let mut dest = tokio::fs::File::create(&target_path).await
        .map_err(|e| napi::Error::from_reason(format!("Disk error: {}", e)))?;

    dest.write_all(&bytes).await
        .map_err(|e| napi::Error::from_reason(format!("Write failed: {}", e)))?;

    // Ensure the data is flushed to the SSD before returning to Electron
    dest.sync_all().await
        .map_err(|e| napi::Error::from_reason(format!("Flush failed: {}", e)))?;

    Ok(target_path)
}