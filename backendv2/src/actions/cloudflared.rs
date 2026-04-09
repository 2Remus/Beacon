use std::env;
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

lazy_static! {
    static ref ACTIVE_TUNNEL: Mutex<Option<Child>> = Mutex::new(None);
}


/// Helper to locate the binary based on OS
fn get_cloudflared_path(resource_dir: &Path) -> PathBuf {
    if cfg!(windows) {
        resource_dir.join("bin").join("cloudflared.exe")
    } else {
        // MacOS/Linux
        resource_dir.join("bin").join("cloudflared")
    }
}




#[napi]
pub async fn  start_cloudflared(port: u32) -> napi::Result<cloudflaredRespone>{

    let resource_dir = get_resource_path()?;
    let cloudflared_bin = get_cloudflared_path(&resource_dir);

    if !cloudflared_bin.exists() {
        return Err(Error::from_reason(format!("cloudflared {} is not exist", cloudflared_bin.display())));
    }

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
        .map_err(|e| Error::from_reason(format!("Failed to launch: {}", e)))?;


    let stderr = child.stderr.take().ok_or_else(|| Error::from_reason("Failed to capture stderr"))?;
    let reader = BufReader::new(stderr);

    let mut detected_url = String::from("pending");

    // Scan lines until we find the URL
    for line in reader.lines() {
        // Explicitly annotate 'l' as a String
        if let Ok(l) = line {
            let l: String = l; // This "shadows" the variable with an explicit type

            println!("{}", l);

            if l.contains(".trycloudflare.com") {
                // Help the compiler with the split/last operations as well
                if let Some(url) = l.split("at ").collect::<Vec<&str>>().last() {
                    detected_url = url.trim().to_string();
                    break;
                }
            }

            if l.contains("failed to request quick Tunnel") {
                return Err(Error::from_reason("Cloudflare timeout or connection error"));
            }
        }
    }

    // Store the child process so it doesn't get dropped (killing the tunnel)
    let mut lock = ACTIVE_TUNNEL.lock().unwrap();
    *lock = Some(child);

    Ok(cloudflaredRespone {
        url: detected_url,
        status: "RUNNING".to_string(),
        connections: vec!["Cloudflare Edge".to_string()],
    })
}
#[napi]
pub fn stop_cloudflared(env: Env) -> napi::Result<(String)> {
    let mut lock = ACTIVE_TUNNEL.lock().unwrap();

    if let Some(mut child) = lock.take() {
        child.kill().map_err(|e| napi::Error::from_reason(format!("{:?}", e)))?;

        let _ = child.wait();

        Ok("Tunnel stopped".to_string())

    }else{
        Ok("No active tunnel found to stop.".to_string())

    }

}



#[napi]
pub fn client_connect(url: String) -> napi::Result<String> {
    let resource_dir = get_resource_path()?;
    let cloudflared_bin = get_cloudflared_path(&resource_dir);

    // ... (Your binary existence check here) ...

    let mut child = Command::new(&cloudflared_bin)
        .args(&[
            "tunnel",
            "--url",
            &url,
            "--no-autoupdate",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| Error::from_reason(format!("Failed to launch: {}", e)))?;

    let connection_id = format!("tunnel-{}", url);

    // --- MUTEX INSERTION ---
    {
        // 1. Acquire the lock. .map_err converts a poisoned lock error into a JS error.
        let mut registry = ACTIVE_TUNNEL.lock().map_err(|_| {
            Error::from_reason("Failed to lock ACTIVE_TUNNEL")
        })?;

        // Instead of .insert(), just wrap the child in Some()
        *registry = Some(child);

        // The lock is automatically released here when 'registry' goes out of scope
    }

    println!("[Rust] Started tunnel for: {}", url);
    Ok(connection_id)
}
