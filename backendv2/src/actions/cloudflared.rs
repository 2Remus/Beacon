use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::process::Child;
use std::sync::Mutex;
use lazy_static::lazy_static;
use crate::get_resource_path;
use crate::models::Cloudflared::cloudflaredRespone;
use glob::glob;

lazy_static! {
    static ref ACTIVE_TUNNEL: Mutex<Option<Child>> = Mutex::new(None);
}


/// Helper to locate the binary based on OS
fn get_cloudflared_path(resource_dir: &Path) -> Option<PathBuf> {
    let pattern = if cfg!(windows) {
        "bin/cloudflared*.exe"
    }
    else{
        "bin/cloudflared"
    };

    let full_pattern = resource_dir.join(pattern);
    let pattern_str = full_pattern.to_str()?;

    if let Ok(paths) = glob(pattern_str) {
        for entry in paths {
            if let Ok(path) = entry {
                return Some(path)
            }
        }
    }

    None
}


pub async fn get_bin(data_dir: String) -> napi::Result<String> {
    let data_path = std::path::PathBuf::from(data_dir.clone());
    let resource_dir = get_resource_path(data_dir.clone())?;

    if let Some(path) = get_cloudflared_path(&resource_dir) {
        return Ok(path.to_string_lossy().into_owned());
    }

    let bin_name = if cfg!(windows) { "cloudflared.exe" } else { "cloudflared" };
    let target_path = data_path.join("bin").join(bin_name);

    // Ensure the bin directory exists
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| napi::Error::from_reason(e.to_string()))?;
    }

    let client = reqwest::Client::new();

    // 3. Fetch Release Info
    let _release_info = client
        .get("https://api.github.com/repos/cloudflare/cloudflared/releases/latest")
        .header("User-Agent", "Beacon-App")
        .send()
        .await
        .map_err(|e| napi::Error::from_reason(e.to_string()))?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    // 4. Determine URL (Simplified for brevity)
    let download_url = if cfg!(windows) {
        "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-windows-amd64.exe"
    } else {
        "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64"
    };

    // 5. Perform the actual download
    let response = client.get(download_url)
        .send()
        .await
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    let content = response.bytes().await
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    std::fs::write(&target_path, &content)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    // 6. On Unix, we MUST make the file executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&target_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&target_path, perms).unwrap();
    }

    Ok(target_path.to_string_lossy().into_owned())
}


#[napi]
pub async fn start_cloudflared(port: u32, data_dir: String) -> napi::Result<cloudflaredRespone> {
    //let resource_dir = std::path::PathBuf::from(data_dir);

    // FIX 1: Use ok_or_else for Option -> Result conversion
    let cloudflared_bin = get_bin(data_dir.clone()).await?;

    // FIX 2: spawn() returns a Result. Map the error THEN use '?'
    let mut child = Command::new(&cloudflared_bin)
        .args(&[
            "tunnel",
            "--url",
            &format!("http://localhost:{}", port),
            "--no-autoupdate",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    // FIX 3: .stderr.take() returns an Option. Use ok_or_else.
    let stderr = child.stderr.take()
        .ok_or_else(|| napi::Error::from_reason("Failed to capture stderr pipe"))?;

    let reader = BufReader::new(stderr);
    let mut detected_url = String::from("pending");

    for line in reader.lines() {
        if let Ok(l) = line {
            if l.contains(".trycloudflare.com") {
                if let Some(url) = l.split_whitespace().find(|&s| s.contains(".trycloudflare.com")) {
                    detected_url = url.trim().replace("url=", "").to_string();
                    break;
                }
            }
            if l.contains("failed to request quick Tunnel") {
                let _ = child.kill();
                return Err(napi::Error::from_reason("Cloudflare tunnel request failed"));
            }
        }
    }

    let mut lock = ACTIVE_TUNNEL.lock().unwrap();
    *lock = Some(child);

    Ok(cloudflaredRespone {
        url: detected_url,
        status: "RUNNING".to_string(),
        connections: vec!["Cloudflare Edge".to_string()],
    })
}
#[napi]
pub fn stop_cloudflared(_env: Env) -> napi::Result<String> {
    let mut lock = ACTIVE_TUNNEL.lock().unwrap();

    if let Some(mut child) = lock.take() {
        child.kill().map_err(|e| napi::Error::from_reason(format!("{:?}", e)))?;

        let _ = child.wait();

        Ok("Tunnel stopped".to_string())

    }else{
        Ok("No active tunnel found to stop.".to_string())

    }

}



// #[napi]
// pub async fn client_connect(url: String, data_dir: String) -> napi::Result<String> {
//     let resource_dir = get_resource_path(data_dir.clone())?;
//     let cloudflared_bin = get_bin(data_dir.clone()).await?;
    
//     let mut child = Command::new(&cloudflared_bin)
//         .args(&["tunnel", "--url", &url, "--no-autoupdate"])
//         .stdout(Stdio::piped())
//         .stderr(Stdio::piped())
//         .spawn();

//     // Store in the same Mutex (Note: this will replace any server tunnel currently running)
//     let mut registry = ACTIVE_TUNNEL.lock().unwrap();
//     *registry = Some(child?);

//     Ok(format!("tunnel-{}", url))
// }




//new client connect implementation
#[napi]
pub async fn client_connect(url: String, data_dir:String) -> napi::Result<String> {
    let cloudflared_bin = get_bin(data_dir.clone()).await?;


    
        let mut registry = ACTIVE_TUNNEL.lock().map_err(|_| {
            napi::Error::from_reason("Failed to lock tunnel registry")
        })?;
        
        if let Some(mut old_child) = registry.take() {
            let _ = old_child.kill(); // Kill previous if it exists
        } 
    

    let child = Command::new(&cloudflared_bin)
        .args(&["tunnel", "--url", &url, "--no-autoupdate"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| napi::Error::from_reason(format!("Failed to spawn tunnel: {}", e)))?;

    // 3. Store the new child
    let mut registry = ACTIVE_TUNNEL.lock().unwrap();
    *registry = Some(child);

    Ok(format!("tunnel-{}", url))

}
