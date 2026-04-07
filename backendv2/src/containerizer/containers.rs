use napi_derive::napi;

pub struct Container {
    pub id: u32,
    pub name: String,
    pub dir: String,
    pub jar: String,
}


#[napi(object)]
pub struct SpawnResult {
    pub pid: u32,
    pub log_path: String,
}