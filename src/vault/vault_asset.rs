//

//

use std::{path::PathBuf, fs::DirEntry, io::Write};

use bevy::{prelude::Vec2, ecs::system::{Resource, Commands, Res, ResMut}, asset::{AssetServer, Assets, Handle, Asset, AssetLoader, io::{Reader, Writer}, AsyncReadExt, saver::{AssetSaver, SavedAsset}, AsyncWriteExt, LoadContext}, reflect::TypePath, utils::BoxedFuture, text::TextSettings};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{CurrentVault, VaultOfVaults};

pub const VAULTS_FILE_NAME: &str = "karta.vault";

pub fn load_vaults(
    mut _commands: Commands,
    asset_server: Res<AssetServer>,
    _vault_asset_state: Res<VaultAssetState>,
    assets: ResMut<Assets<VaultAsset>>,
    vault: Res<CurrentVault>,
){


}

#[derive(Resource, Default)]
pub struct VaultAssetState {
    pub _handle: Handle<VaultAsset>,
}

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


#[derive(Default)]
pub struct VaultAssetLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum VaultAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could load shader: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

impl AssetLoader for VaultAssetLoader {
    type Asset = VaultAsset;
    type Settings = ();
    type Error = VaultAssetLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_vault: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let asset: VaultAsset = ron::de::from_bytes::<VaultAsset>(&bytes)?;
            Ok(asset)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["vault"]
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

fn save_vault (

) {

}
