use napi_derive::napi;

#[napi(object)]
pub struct cloudflaredRespone {
    pub url: String,
    pub status: String,
    pub connections: Vec<String>,
}