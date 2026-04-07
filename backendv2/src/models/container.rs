use napi_derive::napi;

#[napi(object)]
pub struct ContainerConfig {
    pub server_id: String,
    pub jar_path: String,
    pub port: u32,
    pub ram_mb: u32,
    pub enable_rcon: bool,
    pub online_mode: bool,
}
