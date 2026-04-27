use std::env;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use napi::Error;
use napi_derive::napi;
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref ACTIVE_NGINX: Mutex<Option<Child>> = Mutex::new(None);
}


#[napi]
pub fn start_nginx(data_dir: String) -> napi::Result<String> {

    let resource_dir = crate::get_resource_path(data_dir).unwrap();

    //setup paths
    let nginx_bin = if cfg!(windows) {
        resource_dir.join("bin/win32/nginx.exe")
    }
    else{
        resource_dir.join("bin/darwin/nginx")
    };

    let working_dir = if cfg!(windows) {
        PathBuf::from(env::var("APPDATA").unwrap()).join("Beacon")
    }
    else{
        dirs::data_dir().unwrap().join("Beacon")
    };

    let temp_config_path = working_dir.join("nginx_runtime.conf");

    let child = Command::new(nginx_bin)
        .args(&[
            "-p", &resource_dir.join("bin/win32").to_string_lossy(),
            "-c", &temp_config_path.to_string_lossy()
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| Error::from_reason(format!("Nginx spawn failed: {}", e)));


    let mut lock = ACTIVE_NGINX.lock().unwrap();

    *lock = Some(child?);


    Ok("Nginx started with custom config".to_string())
}


#[napi]
pub fn stop_nginx(data_dir: String) -> napi::Result<String> {
    let resource_dir = crate::get_resource_path(data_dir).unwrap();
    let  mut process = ACTIVE_NGINX.lock().unwrap();
    if let Some(mut child) = process.take() {
        child.kill().map_err(|e| napi::Error::from_reason(format!("{:?}", e)))?;

        let _ = child.wait();

        Ok("Tunnel stopped".to_string())

    }else{
        Ok("No active tunnel found to stop.".to_string())

    }

}

// pub fn new_server() -> napi::Result<String> {
// 
// }
