use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::process::Child;
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref ACTIVE_TUNNEL: Mutex<Option<Child>> = Mutex::new(None);
}


/// Helper to locate the binary based on OS
fn get_cloudflared_path(resource_dir: &Path) -> PathBuf {
    if cfg!(windows) {
        resource_dir.join("bin").join("win32").join("cloudflared.exe")
    } else {
        // MacOS/Linux
        resource_dir.join("bin").join("darwin").join("cloudflared")
    }
}

#[napi]
pub async fn start_cloudflared(port: u32) -> napi::Result<String> {
    // 1. Get the base resource path (from your lib.rs helper)
    let resource_dir = crate::get_resource_path()?;

    // 2. Resolve the binary path
    let cloudflare_bin = get_cloudflared_path(&resource_dir);

    // 3. Verify existence before spawning
    if !cloudflare_bin.exists() {
        return Err(Error::from_reason(format!(
            "Cloudflared binary missing at: {:?}",
            cloudflare_bin
        )));
    }

    // 4. Execute the sidecar
    // We use "spawn" instead of "output" because we want it to run in the background
    let child = Command::new(&cloudflare_bin)
        .args(&[
            "tunnel",
            "--url",
            &format!("http://localhost:{}", port),
            "--no-autoupdate"
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| Error::from_reason(format!("Failed to launch cloudflared: {}", e)))?;

    let mut lock = ACTIVE_TUNNEL.lock().unwrap();

    *lock = Some(child);

    Ok(format!("Tunnel initiated via {:?}", cloudflare_bin))
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


