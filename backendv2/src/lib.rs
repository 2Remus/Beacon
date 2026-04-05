mod models;
mod actions;
mod containerizer;
mod database;

use std::env;
use std::fs;
use std::path::{ PathBuf, Path};
use std::process::{Command, Stdio};
use napi_derive::napi;
use actions::nginx::start_nginx;
pub use actions::server::*;

#[napi]
pub async fn initialize_env() -> napi::Result<()> {

    let resource_dir = get_resource_path()?;
    let dbfile = get_dbjson()?;

    let (nginx_bin, cf_bin) = if cfg!(windows) {
        (resource_dir.join("bin/win32/nginx.exe"), resource_dir.join("bin/win32/cloudflared.exe"))
    }
    else{
        (resource_dir.join("bin/darwin/nginx"),  resource_dir.join("bin/darwin/cloudflared.exe"))

    };



    if !nginx_bin.exists() || !cf_bin.exists(){
        return Err(napi::Error::from_reason("Sidecar binaries missing form app directory"))
    }


    setup_nginx_sidecar(&nginx_bin).map_err(|e| napi::Error::from_reason(format!("{:?}", e)))?;

    let data_json = &dbfile;
    if !data_json.exists(){
        let default_data = r#"{
    "last_updated": "",
    "instances": []
}"#;
        fs::write(dbfile, default_data).map_err(|e| {
            napi::Error::from_reason(format!("Failed to initialize servers.json: {}", e))
        })?;
    }

    Ok(())
}



fn get_resource_path() -> napi::Result<PathBuf> {
    let mut path = env::current_exe()
    .map_err(|e| napi::Error::from_reason(format!("{:?}", e)))?;

    if cfg!(windows) {
        path.pop();
        path.push("resources");

    }
    else {
        path.pop();
        path.pop();
        path.push("Resources");

    }
    Ok(path)
}


fn get_dbjson() -> napi::Result<PathBuf> {
    let mut path = env::current_exe()
        .map_err(|e| napi::Error::from_reason(format!("{:?}", e)))?;
    if cfg!(windows) {
        path.pop();
        path.push("servers.json");
    }
    else{
        path.pop();
        path.pop();
        path.push("servers.json");
    }

    Ok(path)
}



fn setup_nginx_sidecar(nginx_bin: &Path) -> std::io::Result<()>{
    let working_dir = if cfg!(windows) {
        PathBuf::from(env::var("APPDATA").unwrap()).join("Beacon")

    }else {
        dirs::home_dir().unwrap().join("Beacon")

    };

    fs::create_dir_all(&working_dir);
    let conf_path = working_dir.join("Sidecar.conf");
    let log_path = working_dir.join("Sidecar.log");
    let pid_path = working_dir.join("nginx.pid");


    let conf_content = format!(
        r#"
            #temporary config file
            worker_processes  1;
            error_log  "{log_file}" warn;
            pid        "{pid_path}";
            events {{ worker_connections  1024; }}
            http {{
                server {{
                    listen       25565; # Avoid sudo/Admin requirements by using a high port
                    server_name  beacon.local;
                    location / {{
                        proxy_pass http://127.0.0.1:25565;
                    }}
                }}
            }}
        "#,
        log_file = log_path.to_string_lossy().replace("\\", "/"), //take out excess data and normalize string
        pid_path = pid_path.to_string_lossy().replace("//", "/"),
    );

    fs::write(&conf_path, conf_content);


    //starts nginx (not needed here)
    // Command::new(nginx_bin)
    //     .arg("-c")
    //     .arg(&conf_path)
    //     .stdout(Stdio::null())
    //     .stderr(Stdio::null())
    //     .spawn();


    Ok(())

}


#[napi]
pub async fn run_env(){
    start_nginx();
}