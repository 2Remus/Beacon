use napi_derive::napi;

#[napi(object)]
pub struct ContainerConfig {
    pub server_id: String,
    pub jar_path: String,
    pub port: Option<u32>,
    pub ram_mb: Option<u32>,
    pub enable_rcon: Option<bool>,
    pub online_mode: Option<bool>,
}
impl Default for ContainerConfig{
    fn default() -> Self{
        Self{
            server_id: "Test".to_string(),
            jar_path: "/var/lib/minecraft".to_string(),
            port: Some(25565),
            ram_mb: Some(3000),
            enable_rcon: Some(true),
            online_mode: Some(true),
        }
    }
}
