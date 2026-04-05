use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::get_resource_path;
use std::fs;
use std::process::{Child, Command, Stdio};
use std::io::Write;
use napi_derive::napi;
use std::sync::Mutex;   
use lazy_static::lazy_static;
use napi::Error;

lazy_static! {
    // Maps Server ID (e.g., "Survival-01") to the OS Process
    static ref RUNNING_SERVERS: Mutex<HashMap<String, Child>> = Mutex::new(HashMap::new());
}

pub fn create_container_env(
    server_id: &str,
    jar_path: &Path,
    allocated_port: u32 // Added: Defensive networking
) -> napi::Result<PathBuf> {
    let container_dir = get_resource_path()?.join("containers").join(server_id);

    // 1. Structural Isolation
    fs::create_dir_all(&container_dir.join("world"))?;
    fs::create_dir_all(&container_dir.join("logs"))?;
    fs::create_dir_all(&container_dir.join("plugins"))?; // Added: Essential for MC

    // 2. The "Virtual" File Link
    let local_jar = container_dir.join("server.jar");
    if !local_jar.exists() {
        #[cfg(unix)] { std::os::unix::fs::symlink(jar_path, &local_jar)?; }
        #[cfg(windows)] { std::os::windows::fs::symlink_file(jar_path, &local_jar)?; }
    }

    // 3. Injecting the "Environment Variables" (Properties)
    let props = format!("server-port={}\nquery.port={}\nenable-query=true\n", allocated_port, allocated_port);
    fs::write(container_dir.join("server.properties"), props)?;

    // 4. Permission Check (The "Lock")
    fs::write(container_dir.join("eula.txt"), "eula=true")?;

    Ok(container_dir)
}


pub fn spawn_container(container_dir: &Path, ram: u32) -> napi::Result<(Child)>{
    let java_bin = get_resource_path()?.join("runtime/java21/bin/java");

    let child =  Command::new(java_bin)
        .current_dir(container_dir)
        .args(&[
            &format!("-Xmx{}M",  ram),
            &format!("Xms{}M", ram),
            "-jar", "server.jar", "nogui"
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    Ok(child?)
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