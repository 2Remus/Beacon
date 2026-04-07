use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::get_resource_path;
use std::fs;
use std::process::{Child, Command, Stdio};
use std::io::Write;
use std::os::unix::process::CommandExt;
use napi_derive::napi;
use std::sync::Mutex;   
use lazy_static::lazy_static;
use napi::{Error,Result};
use crate::models::container::ContainerConfig;
use crate::containerizer::containers::SpawnResult;



lazy_static! {
    // Maps Server ID (e.g., "Survival-01") to the OS Process
    static ref RUNNING_SERVERS: Mutex<HashMap<String, Child>> = Mutex::new(HashMap::new());
}



pub fn create_container_env(config: ContainerConfig) -> napi::Result<PathBuf> {
    let root = get_resource_path()?.join("containers").join(&config.server_id);
    let jar_source = Path::new(&config.jar_path);

    // 1. Recursive Directory Initialization
    // We create the entire tree in one go to ensure paths exist for symlinks
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

    // 3. Properties Injection (The "Configuration Layer")
    // We use a Map-like approach here to make it easier to add more settings later
    let mut properties = Vec::new();
    properties.push(format!("server-port={}", config.port));
    properties.push(format!("query.port={}", config.port));
    properties.push("server-ip=127.0.0.1".to_string()); // Force local binding for Tunnel security
    properties.push("enable-query=true".to_string());
    properties.push(format!("online-mode={}", config.online_mode));
    properties.push("gui=false".to_string());

    if config.enable_rcon {
        properties.push("enable-rcon=true".to_string());
        properties.push("rcon.password=beacon_secure_pass".to_string());
    }

    fs::write(root.join("server.properties"), properties.join("\n"))?;

    // 4. Legal & Runtime Requirements
    fs::write(root.join("eula.txt"), "eula=true")?;

    // Create a start script helper for the JVM
    let jvm_args = format!(
        "java -Xmx{}M -Xms{}M -jar server.jar nogui",
        config.ram_mb, config.ram_mb
    );
    fs::write(root.join("start.sh"), jvm_args)?;

    Ok(root)
}

#[napi]
pub fn spawn_container(id: String, bin_dir: String, ram: u32) -> Result<SpawnResult> {
    let base_path = Path::new(&bin_dir);

    // 1. Path Guard: Ensure the directory and server.jar actually exist
    if !base_path.exists() {
        return Err(Error::from_reason(format!("Directory not found: {}", bin_dir)));
    }

    let jar_path = base_path.join("server.jar");
    if !jar_path.exists() {
        return Err(Error::from_reason("server.jar missing from instance directory"));
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

    Ok(SpawnResult {
        pid,
        log_path: base_path.join("logs/latest.log").to_string_lossy().into_owned(),
    })
}



pub fn kill_container(server_id: String) -> napi::Result<(String)>{

    let mut servers = RUNNING_SERVERS.lock().map_err(|_| Error::from_reason("Lock failed"))?;

    if let Some(mut child) = servers.remove(&server_id){

        //try to send the stop command to the stdin
        if let Some(mut stdin) = child.stdin.take(){
            let _ = stdin.write_all(b"stop\n");
            let _ = stdin.flush();
        }

        match child.wait() {
            Ok(status) => Ok(format!("Server stopped with status: {}", status)),
            Err(_) => {
                // 3. Fallback: Force Kill if it hangs
                child.kill().map_err(|e| Error::from_reason(format!("Force kill failed: {}", e)))?;
                Ok("Server was force-terminated.".to_string())
            }
        }
    }else{
        Err(Error::from_reason(format!("Server not running: {}", server_id)))
    }
}