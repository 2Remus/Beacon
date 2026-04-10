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
