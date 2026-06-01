use zip::ZipArchive;
use std::path::PathBuf;
use std::fs::File;
use crate::get_resource_path;
use std::fs;
use std::io;
use napi::Error;
use crate::models::ServerStorage::MinecraftServer;
use crate::database::add_server::add_server;



pub async fn server_import(server: &MinecraftServer, _container: PathBuf, data_dir: String, db_path: PathBuf) -> napi::Result<()>{ // Changed T to () since you return Ok(())

    println!("importing server");
	let path = PathBuf::from(server.world.as_deref().unwrap_or("world"));
	let file = File::open(&path).map_err(|e| napi::Error::from_reason(e.to_string()))?;

    // 1. Unpack the ZipArchive Result immediately using map_err
	let mut archive = ZipArchive::new(file)
        .map_err(|e| napi::Error::from_reason(format!("Failed to open zip: {}", e)))?;

	let root = get_resource_path(data_dir)?.join("containers").join(&server.id).join("world");

	if !root.exists(){
        // napi::Error requires a specific return, you can't just return a string
		return Err(napi::Error::from_reason("World folder missing"));
	}

	for i in 0..archive.len() {
		let mut file = archive
		 	.by_index(i)
		 	.map_err(|e| napi::Error::from_reason(format!("Zip index error: {}", e)))?;

		let output = match file.enclosed_name() {
			Some(path) => root.join(path),
			None => continue,
		};

		if file.is_dir() {
			fs::create_dir_all(&output).map_err(|e| napi::Error::from_reason(e.to_string()))?;
		} else {
			if let Some(p) = output.parent() {
				if !p.exists() {
					fs::create_dir_all(&p).map_err(|e| napi::Error::from_reason(e.to_string()))?;
				}
			}
			let mut outfile = fs::File::create(&output).map_err(|e| napi::Error::from_reason(e.to_string()))?;
			io::copy(&mut file, &mut outfile).map_err(|e| napi::Error::from_reason(e.to_string()))?;
		}

		#[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&output, fs::Permissions::from_mode(mode))?;
            }
        }


	};

	println!("imported archive");

	let db_result = add_server(&server, db_path.clone());

	match db_result{
	    Ok(()) => {}
		Err(_) => {
		}
	}

	println!("added to dbjson");
	Ok(())
}
