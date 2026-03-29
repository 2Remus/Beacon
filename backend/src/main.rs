use axum::{routing::get, Router};
use bollard::Docker;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};
use http::{header, Method};
use std::io::{self, Write};
use std::path::PathBuf;

pub mod routes;
pub mod dbmodels;
pub mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // --- CHANGE 1: DYNAMIC PATH RESOLUTION ---
    // Instead of looking for .env in the current working directory (which is
    // inconsistent in Electron), we look for a BEACON_ROOT passed by the frontend.
    let root_dir = env::var("BEACON_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::current_dir().unwrap());

    let env_path = root_dir.join(".env");
    dotenvy::from_path(&env_path).ok();

    println!("--- Project Beacon Control Plane ---");
    println!("🚀 Mode: Native App Sidecar");
    println!("📂 Root Context: {:?}", root_dir);
    io::stdout().flush().unwrap();

    // Fetch Public IP (Keep this, it's great for your P2P logic)
    match reqwest::get("https://api.ipify.org").await {
        Ok(resp) => {
            if let Ok(ip) = resp.text().await {
                println!("🌐 Public IP: {}", ip);
            }
        }
        Err(_) => println!("🌐 Public IP: Offline or Local Only"),
    }

    // --- CHANGE 2: TIGHTENED CORS ---
    // Since the Vue UI is now running inside Electron, it will likely
    // be served from localhost:5173 (Dev) or a custom file protocol (Prod).
    let cors = CorsLayer::new()
        .allow_origin([
            "http://beacon.local".parse()?,
            "http://api.beacon.local".parse()?,
            "http://localhost:5173".parse()?,
        ])
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    // --- CHANGE 3: DATABASE URL SOURCE ---
    // Since we aren't in Docker, DATABASE_URL must point to 127.0.0.1:5432
    // even if the Postgres DB is running inside Docker.
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (should point to localhost:5432 for native run)");

    let keycloak_realm = env::var("KEYCLOAK_REALM")
        .unwrap_or_else(|_| "beacon".to_string());

    // 3. Initialize Database with Retry Logic (Extended for Docker Cold-Starts)
    let mut pool = None;
    let max_retries = 15; // Increased retries since Docker Desktop can be slow to wake up

    for i in 1..=max_retries {
        println!("🔄 Connecting to DB (Attempt {}/{})...", i, max_retries);
        match PgPoolOptions::new()
            .max_connections(20)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&database_url)
            .await
        {
            Ok(p) => {
                // --- CHANGE 4: AUTO-MIGRATIONS ---
                // We no longer expect the user to run setup-db.sh.
                // The API now "owns" its schema.
                println!("⚙️ Running Database Migrations...");
                sqlx::migrate!("./migrations")
                    .run(&p)
                    .await?;

                pool = Some(p);
                break;
            }
            Err(e) => {
                if i == max_retries {
                    eprintln!("❌ DB Connection Permanent Failure: {}", e);
                    anyhow::bail!("Could not connect to database");
                }
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
    let pool = pool.unwrap();

    // 4. Initialize Docker Engine Client
    // Bollard defaults work great on Mac/Windows/Linux native as long as Docker Desktop is running.
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| {
            eprintln!("❌ Docker socket unreachable. Is Docker Desktop running?");
            e
        })?;

    println!("✅ Infrastructure (DB & Docker) Ready");

    // 5. Shared Application State
    let app_state = Arc::new(state::AppState {
        pool,
        docker,
        keycloak_realm,
        //keycloak_config: keycloak::Keycloak::Config::load_from_env(),
    });


    let cors = CorsLayer::new()
        .allow_origin(Any) // For dev, allow everything. In prod, lock this to beacon.local
        .allow_methods(Any)
        .allow_headers(Any);
    
    // 6. Security & Routing
    let app = Router::new()
        .route("/", get(|| async { "Beacon Control Plane API v1.0" }))
        .route("/health", get(|| async { "OK" }))
        .nest("/api/v1", routes::api_router())
        .layer(cors)
        .with_state(app_state);

    // 7. Start the Axum Server
    let port = env::var("BACKEND_PORT").unwrap_or_else(|_| "8000".to_string());
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?; // Bound to localhost for security

    // --- CHANGE 5: IPC SIGNALLING ---
    // We print a very specific string so Electron knows exactly when to
    // hide the loading screen and show the Dashboard.
    println!("READY: BEACON_API_LIVE");
    println!("📡 Listening on {}", addr);
    io::stdout().flush().unwrap();

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}