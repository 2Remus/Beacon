use bollard::Docker;
use sqlx::PgPool;
use std::sync::Arc;
//use crate::keycloak::Keycloak::Config;
//use crate::ollama::OllamaService;

/// The central state object for the application.
/// It is wrapped in an Arc and shared across all Axum handlers.
pub struct AppState {
    pub pool: PgPool,
    //pub redis_client: redis::Client,
    pub docker: Docker,
    pub keycloak_realm: String,
    //pub keycloak_config: Config,
    //pub ollama: Arc<OllamaService>
}