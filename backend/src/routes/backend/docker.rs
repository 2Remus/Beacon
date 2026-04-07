//manages docker container interaction

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
use bollard::container::StartContainerOptions;
use crate::state::AppState;
use bollard::Docker;
use serde_json::json;

fn is_docker_stack_up(root_dir: &PathBuf) -> bool {
    let output = Command::new("docker")
        .current_dir(root_dir)
        .args(&["compose", "ps", "--format", "json"])
        .output();

    if let Ok(out) = output {
        let status_str = String::from_utf8_lossy(&out.stdout);
        // On both OS, if containers are running, the JSON will contain "running"
        return !status_str.trim().is_empty() && status_str.contains("running");
    }
    false
}

fn is_docker_installed() -> bool {
    Command::new("docker").arg("--version").stdout(Stdio::null()).status().map_or(false, |s| s.success())
}
fn update_hosts(aliases: &[&str], add: bool) -> io::Result<()> {
    let path = if cfg!(windows) {
        r"C:\Windows\System32\drivers\etc\hosts"
    } else {
        "/etc/hosts"
    };

    let content = fs::read_to_string(path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    // Remove existing to prevent duplicates
    lines.retain(|line| !aliases.iter().any(|&a| line.contains(a)));

    if add {
        for alias in aliases {
            lines.push(format!("127.0.0.1 {}", alias));
        }
    }

    let new_content = lines.join("\n");

    #[cfg(unix)] {
        // macOS/Linux: Use sudo to write
        let status = Command::new("sudo")
            .arg("sh")
            .arg("-c")
            .arg(format!("echo '{}' > {}", new_content.replace("'", "'\\''"), path))
            .status()?;

        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Sudo failed"));
        }
        // Flush DNS cache on Mac
        let _ = Command::new("sudo").args(&["killall", "-HUP", "mDNSResponder"]).status();
    }

    #[cfg(windows)] {
        // Windows: This will only work if Electron or the user ran the app as Admin
        if let Err(e) = fs::write(path, new_content) {
            println!("SIGNAL: REQUIRES_ADMIN"); // Electron can catch this to show a dialog
            return Err(e);
        }
        // Flush DNS cache on Windows
        let _ = Command::new("ipconfig").arg("/flushdns").status();
    }

    Ok(())
}

fn full_cleanup(aliases: &[&str], compose_path: &PathBuf) {
    let _ = Command::new("docker")
        .args(&["compose", "-f", compose_path.to_str().unwrap(), "down"])
        .stdout(Stdio::inherit())
        .status();
}

fn start_docker_stack(){

}


pub async fn start_docker_services(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    println!("Checking Docker environment...");

    // 1. Check if Docker is even running
    if !is_docker_installed() {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Docker is not installed or not in PATH"}))
        ).into_response();
    }

    // 2. Identify your compose directory (example: using state or env)
    let root_dir = PathBuf::from("./docker-stuff");
    let compose_file = root_dir.join("docker-compose.yml");

    // 3. Check if already running to avoid redundant starts
    if is_docker_stack_up(&root_dir) {
        return (
            axum::http::StatusCode::OK,
            Json(json!({"message": "Services are already running"}))
        ).into_response();
    }

    // 4. Run Docker Compose Up
    // We use tokio::process::Command for async execution if available,
    // but standard Command is fine for short-lived spawns.
    let status = Command::new("docker")
        .current_dir(&root_dir)
        .args(&["compose", "up", "-d"])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("Docker stack started successfully.");

            // 5. Optional: Update hosts file if you have local aliases
            let aliases = ["my-local-app.test", "db.local"];
            if let Err(e) = update_hosts(&aliases, true) {
                println!("Warning: Failed to update hosts file: {}", e);
            }

            (axum::http::StatusCode::OK, Json(json!({"status": "started"}))).into_response()
        }
        Ok(_) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Docker compose failed to start"}))
        ).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()}))
        ).into_response(),
    }
}