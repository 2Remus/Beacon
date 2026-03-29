use std::fs::{self, File};
use std::io::{self, Write, BufRead, BufReader};
use std::process::{Command, Stdio, Child};
use std::path::{PathBuf, Path};
use std::env;
use std::net::UdpSocket;
use tokio::time::{sleep, Duration};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let aliases = vec![
        "beacon.local",
        "app.beacon.local",
        "api.beacon.local",
        "sso.beacon.local"
    ];

    // --- CHANGE 1: RELIABLE PATHING ---
    // Electron passes 'BEACON_ROOT' so we know exactly where docker-compose.yml is.
    let root_dir = env::var("BEACON_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let mut p = env::current_exe().unwrap();
            p.pop(); p
        });

    let compose_path = root_dir.join("docker-compose.yml");
    let env_path = root_dir.join(".env");

    println!("--- Beacon Host Orchestrator ---");
    println!("📂 Root: {:?}", root_dir);

    // --- CHANGE 2: SILENT INSTALLATION ---
    // We check for Docker/Cloudflared, but we let Electron handle the "UX"
    // of showing an install progress bar if they are missing.
    if !is_docker_installed() {
        println!("SIGNAL: INSTALL_DOCKER"); // Electron catches this
        install_docker().await?;
    }

    if !is_cloudflared_installed() {
        println!("SIGNAL: INSTALL_CLOUDFLARED");
        install_cloudflared().await?;
    }

    if !hosts_already_updated(&aliases) {
        println!("📝 Updating system hosts...");
        // This will trigger sudo on Mac/Linux or require Admin on Windows
        update_hosts(&aliases, true)?;
    } else {
        println!("✅ System hosts already configured.");
    }

    // 3. IDEMPOTENT DOCKER LAUNCH
    if !is_docker_stack_up(&root_dir) {
        println!("🐳 Launching Docker Stack...");

        let mut docker_cmd = Command::new("docker");
        docker_cmd.current_dir(&root_dir);
        docker_cmd.args(&["compose", "up", "-d"]);

        if docker_cmd.status()?.success() {
            println!("READY: DOCKER_STACK_UP");
        }
    } else {
        println!("✅ Docker stack is already running.");
        println!("READY: DOCKER_STACK_UP");
    }

    // 2. LOAD ENVIRONMENT
    let env_vars = if env_path.exists() {
        load_env_map(&env_path)
    } else {
        // --- CHANGE 3: AUTO-GENERATE ENV ---
        // Instead of waiting for a shell script, generate defaults if missing.
        let mut defaults = HashMap::new();
        defaults.insert("DATABASE_URL".to_string(), "postgres://user:pass@127.0.0.1:5432/beacon".to_string());
        defaults
    };

    // 3. P2P NAT HOLE PUNCHING
    let _socket = UdpSocket::bind("0.0.0.0:25565").or_else(|_| UdpSocket::bind("0.0.0.0:0"))?;

    // 4. CLOUDFLARE TUNNEL
    println!("☁️ Starting Tunnel...");
    let mut tunnel_process = start_cloudflare_tunnel(25565)?;

    // --- CHANGE 4: CAPTURE TUNNEL URL ---
    // We need to scrape the 'trycloudflared.com' URL from the logs
    // to send it to the Vue frontend.

    // 5. ORCHESTRATION SETUP
    // NOTE: update_hosts will still trigger the sudo-prompt in Electron
    update_hosts(&aliases, true)?;

    // 6. LAUNCH DOCKER STACK
    println!("🐳 Launching Docker Stack...");
    let mut docker_cmd = Command::new("docker");
    docker_cmd.current_dir(&root_dir);

    // We only boot INFRASTRUCTURE now (DB, Nginx, Keycloak)
    docker_cmd.args(&["compose", "up", "-d"]);

    if docker_cmd.envs(&env_vars).status()?.success() {
        // --- CHANGE 5: SUCCESS SIGNAL ---
        // Tell Electron the stack is up so it can launch the API binary
        println!("READY: DOCKER_STACK_UP");
    }

    // --- CHANGE 6: REMOVE THE INTERACTIVE LOOP ---
    // We no longer use stdin/lines.next() because there is no terminal.
    // We keep the process alive and listen for signals.

    let cleanup_aliases = aliases.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let cleanup_compose = compose_path.clone();

    ctrlc::set_handler(move || {
        let refs: Vec<&str> = cleanup_aliases.iter().map(|s| s.as_str()).collect();
        full_cleanup(&refs, &cleanup_compose);
        std::process::exit(0);
    })?;

    // Keep the orchestrator running to manage the tunnel and docker state
    loop {
        sleep(Duration::from_secs(3600)).await;
    }
}

// ... (Helpers remain largely the same, but remove 'pause_and_exit')

// --- HELPERS ---

fn load_env_map(path: &PathBuf) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            let line = line.trim();
            // Ignore comments and empty lines
            if line.is_empty() || line.starts_with('#') { continue; }
            if let Some((key, value)) = line.split_once('=') {
                map.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
    }
    map
}

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

fn start_cloudflare_tunnel(port: u16) -> io::Result<Child> {
    Command::new("cloudflared")
        .args(&["tunnel", "--url", &format!("tcp://localhost:{}", port), "--no-autoupdate"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
}

fn connect_to_tunnel(hostname: &str) -> io::Result<()> {
    Command::new("cloudflared")
        .args(&["access", "tcp", "--hostname", hostname, "--url", "localhost:25565"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    Ok(())
}

fn ensure_compose_exists(path: &PathBuf) -> io::Result<()> {
    if !path.exists() {
        println!("docker compose does not exist");
    }
    Ok(())
}

fn full_cleanup(aliases: &[&str], compose_path: &PathBuf) {
    let _ = Command::new("docker")
        .args(&["compose", "-f", compose_path.to_str().unwrap(), "down"])
        .stdout(Stdio::inherit())
        .status();
}

fn is_admin() -> bool {
    #[cfg(windows)] {
        Command::new("net").arg("session").stdout(Stdio::null()).stderr(Stdio::null()).status().map_or(false, |s| s.success())
    }
    #[cfg(unix)] { true }
}

fn is_docker_installed() -> bool {
    Command::new("docker").arg("--version").stdout(Stdio::null()).status().map_or(false, |s| s.success())
}

async fn install_docker() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")] {
        Command::new("winget").args(&["install", "Docker.DockerDesktop"]).status()?;
    }
    #[cfg(target_os = "macos")] {
        Command::new("brew").args(&["install", "--cask", "docker"]).status()?;
    }
    Ok(())
}


/// Check if aliases exist in hosts file
fn hosts_already_updated(aliases: &[&str]) -> bool {
    let path = if cfg!(windows) {
        r"C:\Windows\System32\drivers\etc\hosts"
    } else {
        "/etc/hosts"
    };

    fs::read_to_string(path)
        .map(|content| aliases.iter().all(|&alias| content.contains(alias)))
        .unwrap_or(false)
}

/// Check if Docker Compose is running
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

/// Update hosts with OS-specific permission handling
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

fn show_permission_error() {
    #[cfg(windows)] { println!("❌ Error: Please 'Run as Administrator'."); }
}
