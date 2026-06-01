mod models;
mod actions;
mod containerizer;
mod database;

use std::path::PathBuf;
pub use actions::server::*;

fn get_resource_path(data_dir: String) -> napi::Result<PathBuf> {
     let mut path = PathBuf::from(&data_dir);
    //.map_err(|e| napi::Error::from_reason(format!("{:?}", e)))?;

    if cfg!(windows) {
        //path.pop();
        path.push("resources");

    }
    else {
        //path.pop();
        //path.pop();
        path.push("Resources");
    }
    Ok(path)
}
