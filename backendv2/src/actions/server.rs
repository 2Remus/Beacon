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
use crate::get_resource_path;
use crate::database::add_server::add_server;
use crate::models::container::ContainerConfig;
use crate::models::ServerStorage::Provider;
use tokio::fs::File as AsyncFile;
use zip::ZipArchive;
use crate::database::server_import::server_import;
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
    data_dir: String,
) -> napi::Result<String> {

    // 1. RESOLVE & DOWNLOAD (The Pull Step)
    //
    // Pulls from cache or hits the API (Paper/Fabric/Vanilla)
    let jar_path = ensure_jar_exists(&provider, &version, data_dir.clone()).await?;

    // 2. BUILD CONFIG (The Environment Metadata)
    // We initialize fully here to avoid the "partially assigned" E0381 error.
    let container_config = ContainerConfig {
        server_id: id.clone(), // Clone here so we can use 'id' again later
        jar_path: jar_path.to_string_lossy().into_owned(),
        port:Some(port),
        ram_mb:Some(ram_mb),
        enable_rcon: Some(true),
        online_mode:Some(online_mode),
    };

    let container_path = create_container_env(container_config, data_dir.clone())?;

    // 4. REGISTER (The Persistence Layer)
    let new_server = MinecraftServer {
        id: id.clone(),
        name,
        version,
        port: Some(port),
        provider: provider, 
        ram: Some(ram_mb),        
        world: None,
        instance_path: container_path.to_string_lossy().to_string(),
        status: "stopped".to_string(),
        ..Default::default()
    };

    let db_path = std::path::PathBuf::from(data_dir).join("db.json");
    add_server(&new_server, db_path)?;

    Ok(format!("Server {} ({}) provisioned at {:?}", id, &new_server.version, container_path))
}


#[napi]
pub async fn get_servers(data_dir: String) -> napi::Result<Vec<MinecraftServer>> {
    let db_path = std::path::PathBuf::from(data_dir).join("db.json");


    if !db_path.exists() {
        // Ensure the directory exists first
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&db_path, "[]")
            .map_err(|e| napi::Error::from_reason(format!("Init failed: {}", e)))?;
    }

        let content = std::fs::read_to_string(&db_path)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        let servers: Vec<MinecraftServer> = serde_json::from_str(&content)
            .map_err(|e| Error::from_reason(e.to_string()))?;

    Ok(servers)
}


async fn ensure_jar_exists(provider: &Provider, version: &str, path: String) -> napi::Result<PathBuf> {
    let cache_dir = get_resource_path(path)?.join("cache");

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
        .user_agent("ProjectBeacon/1.0 (Software Development Project)")
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


#[napi]
async fn import_server(

    id: String,
    name: String,
    version: String,
    provider: Provider,
    online_mode: bool,
    data_dir: String,
    file_path:String,

    ) -> napi::Result<()>{

    // import_path = PathBuf::from_str(file_path);
    // let file = File::Open(file_path)?;
    // let mut world = ZipArchive::new(file);

    let jar_path = ensure_jar_exists(&provider, &version, data_dir.clone()).await?;



    let config = ContainerConfig {
        server_id: id.clone(), // Clone here so we can use 'id' again later
        jar_path: jar_path.to_string_lossy().into_owned(),
        ..Default:: default()
    };

    let container_path = create_container_env(config, data_dir.clone());

    let server = MinecraftServer {
        id: id.clone(),
        name,
        version,
        provider,
        // Using the actual path returned by the provisioner
        instance_path: container_path.clone().unwrap().to_string_lossy().to_string(),
        status: "stopped".to_string(),
        world: Some(file_path),
        ..Default::default()
    };

    server_import(&server, container_path?, data_dir).await?;

    Ok(())

}