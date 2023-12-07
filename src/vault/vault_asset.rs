//

//

use std::io::Write;

use bevy::{ecs::system::Res, reflect::TypePath, asset::Asset};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use super::VaultOfVaults;

pub const VAULTS_FILE_NAME: &str = "karta.vault";

#[derive(Asset, Debug, Serialize, Deserialize, TypePath, Default)]
pub struct VaultAsset {
    #[serde(default = "Vec::new")]
    pub vaults: Vec<VaultSerial>,

}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct VaultSerial {
    pub vault_root_path: String,
}

pub struct VaultSettings {
    //pub vault_root_path: PathBuf,
}

impl Default for VaultSettings {
    fn default() -> Self {
        Self {
            //vault_root_path: PathBuf::from(""),
        }
    }
}

// define a struct and derive `serde::Serialize` for serialization (rust -> ron i this case)
// `serde::Deserialize` for deserialization (ron -> rust in this case)
#[derive(serde::Serialize, serde::Deserialize)]
struct AStruct {
    a: String,
    b: i32,
}

pub fn save_vaults(
    vaults: Res<VaultOfVaults>,
){
    let project_dirs = ProjectDirs::from("com", "Teodosin", "Karta").unwrap();
    let config_dir = project_dirs.config_dir();

    println!("Config dir: {:?}", config_dir);
    let file_name = VAULTS_FILE_NAME;
    let full_path = config_dir.join(file_name);
    println!("Full path: {:?}", full_path);

    // Check if the config dir exists
    if !config_dir.exists() {
        println!("Config dir does not exist");
        // Create the config dir
        std::fs::create_dir(config_dir).expect("Could not create config dir");
    }

    let mut vaults_serial: Vec<VaultSerial> = Vec::new();

    if vaults.vaults.len() == 0 {
        println!("No vaults to save");
        return
    }

    for vault in vaults.vaults.iter() {
        vaults_serial.push(VaultSerial {
            vault_root_path: vault.root.to_str().unwrap().to_string(),
        });
    }

    let asset = VaultAsset {
        vaults: vaults_serial,
    };

    let data = ron::to_string(&asset).unwrap();

    let mut file = std::fs::File::create(full_path).expect("Could not create vaults file");
    file.write_all(data.as_bytes()).expect("Could not write to vaults file");
    
}
