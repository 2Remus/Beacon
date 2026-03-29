use axum::{routing::get, Router};
use bollard::Docker;
use dotenvy;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use http::{header, Method};
use std::io::{self, Write};
use std::path::PathBuf;

pub mod routes;
pub mod dbmodels;
pub mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. DYNAMIC PATH RESOLUTION
    let root_dir = env::var("BEACON_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::current_dir().unwrap());

    let env_path = root_dir.join(".env");
    dotenvy::from_path(&env_path).ok();

    println!("--- Project Beacon Control Plane ---");
    println!("🚀 Mode: Native App Sidecar");
    println!("📂 Root Context: {:?}", root_dir);
    io::stdout().flush().unwrap();

    // 2. PRODUCTION CORS
    let cors = CorsLayer::new()
        .allow_origin([
            "http://beacon.local".parse()?,
            "http://app.beacon.local".parse()?,
            "http://localhost:5173".parse()?, // Keep for dev
        ])
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    // 3. DATABASE INITIALIZATION (With Retries)
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://user:password@127.0.0.1:5436/beacon".to_string());

    let mut pool = None;
    for i in 1..=15 {
        println!("🔄 Connecting to DB (Attempt {}/15)...", i);
        match PgPoolOptions::new()
            .max_connections(20)
            .connect(&database_url)
            .await
        {
            Ok(p) => {
                println!("⚙️ Running Database Migrations...");
                sqlx::migrate!("./migrations").run(&p).await?;
                pool = Some(p);
                break;
            }
            Err(_) => tokio::time::sleep(Duration::from_secs(2)).await,
        }
    }
    let pool = pool.ok_or_else(|| anyhow::anyhow!("DB Connection Permanent Failure"))?;

    // 4. DOCKER INITIALIZATION
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| {
            eprintln!("❌ Docker unreachable. Ensure Docker Desktop is active.");
            e
        })?;

    // 5. START SERVER
    let app_state = Arc::new(state::AppState {
        pool,
        docker,
        keycloak_realm: env::var("KEYCLOAK_REALM").unwrap_or_else(|_| "beacon".to_string()),
    });

    let app = Router::new()
        .route("/", get(|| async { "Beacon Control Plane API v1.0" }))
        .nest("/api/v1", routes::api_router())
        .layer(cors)
        .with_state(app_state);

    let port = env::var("BACKEND_PORT").unwrap_or_else(|_| "8000".to_string());
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;

    // CRITICAL: This exact string tells Electron to close the splash screen
    println!("READY: BEACON_API_LIVE");
    println!("📡 Listening on {}", addr);
    io::stdout().flush().unwrap();

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}