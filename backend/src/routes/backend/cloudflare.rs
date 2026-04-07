//setup cloudflare tunnel etc
use std::fs::{self, File};
use std::io::{self, Write, BufRead, BufReader};
use std::process::{Command, Stdio, Child};
use std::path::{PathBuf, Path};
use std::env;
use std::net::UdpSocket;
use tokio::time::{sleep, Duration};
use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::State;
use axum::Json;
use axum::response::IntoResponse;
use crate::dbmodels::server::startServerRequest;
use crate::state::AppState;
use tokio::task;

fn is_cloudflared_installed() -> bool {
    Command::new("cloudflared").arg("--version").stdout(Stdio::null()).stderr(Stdio::null()).status().map_or(false, |s| s.success())
}
async fn install_cloudflared() -> io::Result<()> {
    #[cfg(target_os = "windows")] {
        Command::new("winget").args(&["install", "--id", "Cloudflare.cloudflared", "--silent"]).status()?;
    }
    #[cfg(target_os = "macos")] {
        Command::new("brew").args(&["install", "cloudflare/cloudflare/cloudflared"]).status()?;
    }
    Ok(())
}





pub async fn start_cloudflare(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    println!("Starting Cloudflare...");

    // 1. Check/Install (Note: install_cloudflared should probably be checked outside the request loop)
    if !is_cloudflared_installed() {
        if let Err(e) = install_cloudflared().await {
            return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Install failed: {}", e)).into_response();
        }
    }

    // 2. Run the blocking tunnel startup in a separate thread
    let result = task::spawn_blocking(move || {
        start_cloudflare_tunnel()
    }).await;

    match result {
        Ok(Ok((child, tunnel_url))) => {
            println!("Tunnel live at: {}", tunnel_url);

            // 3. IMPORTANT: Store the child process in state so it doesn't close
            // state.tunnels.lock().unwrap().insert(payload.server_id, child);

            (axum::http::StatusCode::OK, Json(serde_json::json!({ "url": tunnel_url }))).into_response()
        }
        _ => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Failed to start tunnel").into_response(),
    }
}

fn start_cloudflare_tunnel() -> io::Result<(Child, String)> {
    let mut child = Command::new("cloudflared")
        .args(&[
            "tunnel",
            "--url",
            "tcp://localhost:25565", // Changed to TCP for Minecraft/standard 25565 use
            "--no-autoupdate",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;

    let stderr = child.stderr.take().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to open stderr"))?;
    let reader = BufReader::new(stderr);

    for line in reader.lines() {
        let line_text = line?;
        // cloudflared logs the URL to stderr
        if line_text.contains(".trycloudflare.com") {
            let parts: Vec<&str> = line_text.split_whitespace().collect();
            for part in parts {
                if part.starts_with("https://") {
                    // Clean up potential formatting characters like pipes '|'
                    let clean_url = part.trim_matches('|').trim().to_string();
                    return Ok((child, clean_url));
                }
            }
        }
    }

    Err(io::Error::new(io::ErrorKind::Other, "Tunnel URL not found in logs"))
}