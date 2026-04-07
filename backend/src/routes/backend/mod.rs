mod docker;
mod cloudflare;

use std::sync::Arc;
use crate::state::AppState;
use axum::{routing::get, Router, routing::post};
use crate::routes::server::handlers;

pub fn router() -> Router<Arc<AppState>>{
    Router::new()
        .route("/start", get(cloudflare::start_cloudflare))
}