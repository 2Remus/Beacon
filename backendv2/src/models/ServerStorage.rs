use serde::{Deserialize, Serialize};
use napi_derive::napi;
use strum_macros::Display;

// fn get_app_data_path() -> PathBuf {
//     #[cfg(target_os = "windows")]{
//         Pathbuf::from(env::var("APPDATA").unwrap()).join("Beacon")
//     }
//     #[cfg(target_os = "macos")]{
//         let home = env::var("HOME").unwrap();
//             PathBuf::from(&home)
//                 .join("Library")
//                 .join("Application Support")
//                 .join("Beacon")
//     }
// }



#[napi(string_enum)]
#[derive(Serialize, Deserialize, Debug, Display)]
#[strum(serialize_all = "lowercase")]
pub enum Provider{
    Vanilla,
    Paper,
    Fabric,
    Forge,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
#[napi(object)]
pub struct MinecraftServer {
    pub id: String,
    pub name: String,
    pub instance_path: String,
    pub version: String,
    pub status: String,
    pub provider: Provider,
    pub world: Option<String>,
    pub port: Option<u32>,
    pub ram: Option<u32>,
}

impl Default for MinecraftServer {
    fn default() -> Self {
        Self {
            id: "Test".to_string() ,
            name: "New Server".into(),
            status: "stopped".into(),
            version: "latest".into(),
            instance_path: "/var/lib/minecraft".to_string(),
            provider: Provider::Vanilla, 
            port: Some(25565),
            ram: Some(2048),
            world: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerRegistry {
    pub instances: Vec<MinecraftServer>,
}



#[napi]
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateServerRequest{
    id: String,
    name: String,
    provider: Provider,
    version: String,
    ram_mb: u32,
    port: u32,
}




// impl ServerRegistry {
//     pub fn load() -> Self {
//         let path = get_app_data_path().join("servers.json");
//         let data = fs::read_to_string(path).unwrap_or_else(|_| r#"{"instances": []}"#.to_string());
//         serde_json::from_str(&data).unwrap()
//     }
// 
//     pub fn save(&self) -> std::io::Result<()> {
//         let path = get_app_data_path().join("servers.json");
//         let data = serde_json::to_string_pretty(self)?;
//         fs::write(path, data)
//     }
// }