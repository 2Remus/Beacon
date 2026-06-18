use crate::get_resource_path;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::{fs, thread};

use crate::actions::connections::p2p_host;
use crate::containerizer::containers::SpawnResult;
use crate::models::container::ContainerConfig;
use dashmap::DashMap;
use lazy_static::lazy_static;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi::{Error, Result};
use napi_derive::napi;
use std::sync::Mutex;

#[cfg(unix)]
use std::os::unix::process::CommandExt;

lazy_static! {
    static ref RUNNING_SERVERS: Mutex<HashMap<String, Child>> = Mutex::new(HashMap::new());
    pub static ref PROCESS_REGISTRY: DashMap<String, Child> = DashMap::new();
}

pub fn create_container_env(config: ContainerConfig, path: String) -> napi::Result<PathBuf> {
    let root = get_resource_path(path)?
        .join("containers")
        .join(&config.server_id);
    let jar_source = Path::new(&config.jar_path);

    if !root.exists() {
        fs::create_dir_all(&root)?;
    }

    let dirs = ["world", "logs", "plugins", "mods", "config"];
    for dir in &dirs {
        fs::create_dir_all(root.join(dir))
            .map_err(|e| Error::from_reason(format!("FS Error: {}", e)))?;
    }

    let target_jar = root.join("server.jar");
    if !target_jar.exists() {
        #[cfg(unix)]
        std::os::unix::fs::symlink(jar_source, &target_jar)
            .map_err(|e| Error::from_reason(format!("Symlink failed: {}", e)))?;

        #[cfg(windows)]
        std::os::windows::fs::symlink_file(jar_source, &target_jar)
            .map_err(|e| Error::from_reason(format!("Symlink failed: {}", e)))?;
    }

    let mut properties = Vec::new();
    properties.push(format!("server-port={}", config.port.unwrap_or(25565)));
    properties.push(format!("query.port={}", config.port.unwrap_or(25565)));
    properties.push("server-ip=127.0.0.1".to_string()); // Force local binding for Tunnel security
    properties.push("enable-query=true".to_string());
    properties.push(format!(
        "online-mode={}",
        config.online_mode.unwrap_or(false)
    )); //can cause problems should fix later
    properties.push("gui=false".to_string());

    if config.enable_rcon.unwrap_or(true) {
        properties.push("enable-rcon=true".to_string());
        properties.push("rcon.password=beacon_secure_pass".to_string());
    }

    fs::write(root.join("server.properties"), properties.join("\n"))?;

    // 4. Legal & Runtime Requirements
    fs::write(root.join("eula.txt"), "eula=true")?;

    // Create a start script helper for the JVM
    let jvm_args = format!(
        "java -Xmx{}M -Xms{}M -jar server.jar nogui", //for some reason opens a terminal on
        //windows??
        config.ram_mb.unwrap_or(1024),
        config.ram_mb.unwrap_or(1024)
    );
    fs::write(root.join("start.sh"), jvm_args)?;

    Ok(root)
}

#[napi]
pub fn spawn_container(id: String, bin_dir: String, ram: u32) -> Result<SpawnResult> {
    let base_path = Path::new(&bin_dir);

    // 1. Path Guard: Ensure the directory and server.jar actually exist
    if !base_path.exists() {
        return Err(Error::from_reason(format!(
            "Directory not found: {}",
            bin_dir
        )));
    }

    let jar_path = base_path.join("server.jar");
    if !jar_path.exists() {
        return Err(Error::from_reason(
            "server.jar missing from instance directory",
        ));
    }

    // 2. Command Construction: Explicitly call 'java'
    // We use absolute paths where possible to avoid working directory ambiguity
    let mut cmd = Command::new("java");

    cmd.current_dir(base_path)
        .args(&[
            &format!("-Xmx{}M", ram),
            &format!("-Xms{}M", ram),
            "-Dfile.encoding=UTF-8",
            "-jar",
            "server.jar",
            "nogui",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    p2p_host();
    // --- Unix (macOS & Linux) Attachment ---
    #[cfg(unix)]
    {
        // Sets the process group ID to the child's PID.
        // This allows us to kill the whole group later.
        cmd.process_group(0);
    }

    let child = cmd.spawn().map_err(|e| Error::from_reason(e.to_string()))?;

    // 3. Metadata Return
    // We return the PID so the Vue frontend can map this to the UI 'Active' state
    let pid = child.id();

    PROCESS_REGISTRY.insert(id.clone(), child);

    Ok(SpawnResult {
        pid,
        log_path: base_path
            .join("logs/latest.log")
            .to_string_lossy()
            .into_owned(),
    })
}

#[napi]

//needs to be studied
pub fn stream_logs(id: String, callback: ThreadsafeFunction<String>) -> napi::Result<()> {
    // 1. Get the specific server from the parallel registry
    let mut registry_entry = PROCESS_REGISTRY
        .get_mut(&id)
        .ok_or_else(|| napi::Error::from_reason(format!("Server {} not found", id)))?;

    // 2. Take the stdout handle (ensuring we don't try to take it twice)
    let stdout = registry_entry
        .value_mut()
        .stdout
        .take()
        .ok_or_else(|| napi::Error::from_reason("Stdout already attached or missing"))?;

    // 3. Spawn a "Watcher Thread" just for this ID
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(content) = line {
                // Push to Electron with the content
                callback.call(Ok(content), ThreadsafeFunctionCallMode::Blocking);
            } else {
                break; // Stream closed (server stopped)
            }
        }
    });

    Ok(())
}

#[napi]
pub fn kill_containers() -> napi::Result<String> {
    let count = 0;

    // --- STEP 1: Get keys and drop the iterator immediately ---
    let keys: Vec<String> = {
        // The iterator is created and dropped inside this block
        PROCESS_REGISTRY.iter().map(|r| r.key().clone()).collect()
    };

    // --- STEP 2: Move processes out of the Map ---
    let mut processes = Vec::new();
    for key in keys {
        // remove() is safe here because the iterator lock above is gone
        if let Some((_, child)) = PROCESS_REGISTRY.remove(&key) {
            processes.push(child);
        }
    }

    // --- STEP 3: Kill the processes ---
    for mut child in processes {
        if let Some(mut stdin) = child.stdin.take() {
            // Use write_all but don't let it hang the whole function
            let _ = stdin.write_all(b"stop\n");
            let _ = stdin.flush();
        }

        // Send the SIGKILL signal
        match child.kill() {
            Ok(()) => return Ok("Killed container".to_string()),
            Err(e) => eprintln!("Failed to kill process: {}", e),
        }
    }

    let msg = format!("Successfully terminated {} active server instances.", count);
    println!("{}", msg); // Print to terminal/console for debugging
    Ok(msg)
}

#[napi]
pub async fn kill_container(id: String, _force: bool) -> napi::Result<String> {
    let process = PROCESS_REGISTRY.get_mut(&id);

    if let Some(mut child) = process {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(b"stop\n");
            let _ = stdin.flush();
        }

        match child.kill() {
            Ok(_) => return Ok("Killed container".to_string()),
            Err(e) => eprintln!("Failed to kill process: {}", e),
        }
    }

    let message = format!("Successfully terminated {} active server instances.", id);
    Ok(message)
}
