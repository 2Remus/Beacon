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

#[napi]
pub async fn create_server(
    id: String,
    name: String,
    provider: String, // "paper", "vanilla", "fabric"
    version: String,
    ram_mb: u32,
    port: u32
) -> napi::Result<String> {

    // 1. RESOLVE & DOWNLOAD (The Pull Step)
    // We check the cache first. If missing, we hit the Manifest API.
    let jar_path = ensure_jar_exists(&provider, &version).await?;

    // 2. PROVISION (The Container Step)
    // This creates the /containers/id folder and the symlink to the jar we just got.
    let container_path = create_container_env(&id, &jar_path, port)?;

    // 3. REGISTER (The Database Step)
    let new_server = MinecraftServer {
        id: id.clone(),
        name: name.clone(),
        version: version.clone(),
        server_type: provider.clone(),
        port,
        ram: ram_mb,
        instance_path: container_path.to_string_lossy().to_string(),
        status: "stopped".to_string(),
    };

    let db_path = crate::get_resource_path()?.join("db.json");
    add_server(new_server, db_path)?;

    Ok(format!("Server {} ({}) ready!", id, version))
}


#[napi]
pub fn get_servers(callback: ThreadsafeFunction<Vec<String>>) -> napi::Result<()> {
    let db_path = env::current_exe().unwrap().parent().unwrap().join("db.json");

    // Correct syntax: std::thread::spawn
    std::thread::spawn(move || {
        let mut last_content = String::new();
        loop {
            if let Ok(content) = std::fs::read_to_string(&db_path) {
                if content != last_content {
                    last_content = content.clone();
                    if let Ok(servers) = serde_json::from_str::<Vec<String>>(&content) {
                        // This pushes data to your Electron Dashboard
                        callback.call(Ok(servers), ThreadsafeFunctionCallMode::Blocking);
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    });

    Ok(())
}


async fn ensure_jar_exists(provider: &str, version: &str) -> napi::Result<PathBuf> {
    let cache_dir = crate::get_resource_path()?.join("cache");

    // Ensure the cache directory exists before we try to save files to it
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir).map_err(|e| napi::Error::from_reason(e.to_string()))?;
    }

    let jar_name = format!("{}-{}.jar", provider, version);
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
        "paper" => {
            // 1. Get version builds from the new Fill v3 API
            let api_url = format!("https://fill.papermc.io/v3/projects/paper/versions/{}", version);
            let resp: Value = client.get(api_url).send().await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?.json().await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;

            let build_id = resp["builds"].as_array()
                .and_then(|b| b.last())
                .and_then(|last| last.as_u64())
                .ok_or_else(|| napi::Error::from_reason("No builds found for this version"))?;

            // 2. Get specific build info
            let build_url = format!("https://fill.papermc.io/v3/projects/paper/versions/{}/builds/{}", version, build_id);
            let build_info: Value = client.get(build_url).send().await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?.json().await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;

            let file_name = build_info["downloads"]["server:default"]["name"].as_str()
                .ok_or_else(|| napi::Error::from_reason("Failed to find filename"))?;

            // 3. Construct final download link
            format!(
                "https://fill-data.papermc.io/v3/projects/paper/versions/{}/builds/{}/downloads/{}",
                version, build_id, file_name
            )
        },
        "vanilla" => {
            let manifest: Value = client
                .get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
                .send()
                .await
                .map_err(|e| napi::Error::from_reason(format!("Network request failed: {}", e)))?
                .json()
                .await
                .map_err(|e| napi::Error::from_reason(format!("Failed to parse JSON: {}", e)))?;

            let version_entry = manifest["versions"].as_array()
                .and_then(|v| v.iter().find(|entry| entry["id"] == version))
                .ok_or_else(|| napi::Error::from_reason("Version not found"))?;

            let metadata: Value = client
                .get(version_entry["url"].as_str().unwrap())
                .send()
                .await
                .map_err(|e| napi::Error::from_reason(format!("Network error: {}", e)))? // Step 1: Send
                .json()
                .await
                .map_err(|e| napi::Error::from_reason(format!("JSON error: {}", e)))?;  // Step 2: Parse

            metadata["downloads"]["server"]["url"].as_str()
                .ok_or_else(|| napi::Error::from_reason("No server download found"))?
                .to_string()
        },
        "forge" => {
            // Note: This downloads the installer. We'll handle execution below.
            format!(
                "https://maven.minecraftforge.net/net/minecraftforge/forge/{0}/forge-{0}-installer.jar",
                version
            )
        },
        _ => return Err(napi::Error::from_reason("Unsupported provider")),
    };

    // --- EXECUTE DOWNLOAD ---
    let response = client.get(&download_url).send().await
        .map_err(|e| napi::Error::from_reason(format!("Download request failed: {}", e)))?;

    let mut content =  Cursor::new(response.bytes().await
        .map_err(|e| napi::Error::from_reason(format!("Failed to read bytes: {}", e)))?);

    let mut dest = File::create(&target_path)
        .map_err(|e| napi::Error::from_reason(format!("Failed to create jar file: {}", e)))?;

    std::io::copy(&mut content, &mut dest).map_err(|e| napi::Error::from_reason(e.to_string()))?;


    if provider == "forge" {
        let temp_install_dir = cache_dir.join(format!("forge-temp-{}", version));
        fs::create_dir_all(&temp_install_dir)?;

        let status = Command::new("java")
            .arg("-jar")
            .arg(&target_path)
            .arg("--installServer")
            .current_dir(&temp_install_dir)
            .status()
            .map_err(|e| napi::Error::from_reason(format!("Java not found or Forge install failed: {}", e)))?;

        if status.success() {
            println!("Forge installed successfully in {:?}", temp_install_dir);
        }
    }

    Ok(target_path)
}