//

//

use std::{path::PathBuf, fs::DirEntry};

use bevy::{prelude::Vec2, ecs::system::{Resource, Commands, Res, ResMut}, asset::{AssetServer, Assets, Handle, Asset, AssetLoader, io::{Reader, Writer}, LoadVault, AsyncReadExt, saver::{AssetSaver, SavedAsset}, AsyncWriteExt}, reflect::TypePath, utils::BoxedFuture, text::TextSettings};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{CurrentVault, VaultOfVaults};

const VAULTS_FILE_NAME: &str = "karta.vault";

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

#[derive(Asset, Debug, Deserialize, TypePath, Default)]
pub struct VaultAsset {
    #[serde(default = "Vec::new")]
    pub vaults: Vec<VaultSerial>,

}

#[derive(Debug, Deserialize, Default)]
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
        _load_vault: &'a mut LoadVault,
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

// fn save_vault(
//     vaults: Res<VaultOfVaults>,
// ){
//     let vaults_serial;

//     for vault in vaults.vaults.iter() {
//         vaults_serial.push(VaultSerial {
//             vault_root_path: vault.root.to_str().unwrap().to_string(),
//         });
//     }
//     let vaults = vec![VaultSerial {
//             vault_root_path: vault.vault_root_path.to_str().unwrap().to_string(),
//         }];

//     let data = ron::to_string(&vaults).unwrap();


//     writer.write_all(data.as_bytes()).unwrap();
// }
