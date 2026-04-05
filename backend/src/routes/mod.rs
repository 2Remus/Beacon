use axum::Router;
use std::sync::Arc;
use crate::state::AppState;

mod backend;
mod server;


/// The central API router for Beacon v2.
///
/// This function aggregates all sub-routers and ensures they are 
/// strictly typed to receive the Arc<AppState>.
pub fn api_router() -> Router<Arc<AppState>> {
    Router::new()
        // Auth endpoints: /api/v1/auth/login, /api/v1/auth/callback, etc.
        // .nest("/auth", auth::create_router())
        .nest("/servers", server::create_router())
        .nest("/cloudflare",backend::router())

    // Placeholder for future Minecraft instance management
    // .nest("/instances", instances::router())

    // Placeholder for server statistics/monitoring
    // .nest("/stats", stats::router())
}