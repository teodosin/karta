//

//

use std::{path::PathBuf, fs::DirEntry};

use bevy::{prelude::Vec2, ecs::system::{Resource, Commands, Res, ResMut}, asset::{AssetServer, Assets, Handle, Asset, AssetLoader, io::{Reader, Writer}, LoadVault, AsyncReadExt, saver::{AssetSaver, SavedAsset}}, reflect::TypePath, utils::BoxedFuture, text::TextSettings};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::CurrentVault;

pub fn load_vaults(
    mut _commands: Commands,
    asset_server: Res<AssetServer>,
    _vault_asset_state: Res<VaultAssetState>,
    assets: ResMut<Assets<VaultAsset>>,
    vault: Res<CurrentVault>,
){
    let file_name = "thingy.Vault";

    let vault = match &vault.vault {
        Some(vault) => vault,
        None => {
            println!("No vault set");
            return
        }
    };
    
    let path: PathBuf = vault.get_vault_path().join(file_name);
    let vault_assets: Handle<VaultAsset> = 
        asset_server.load(path);
    println!("Vault_assets: {:?}", vault_assets);

    let _data = assets.get(&vault_assets);
    // commands.insert_resource(VaultAssets {
    //     handle: Vault_assets,
    // });
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

struct VaultAssetSaver;

#[derive(Default, Serialize, Deserialize)]
pub struct VaultAssetSaverSettings {
    appended: String,
}

impl AssetSaver for VaultAssetSaver {
    type Asset = VaultAsset;
    type Settings = VaultAssetSaverSettings;
    type OutputLoader = VaultAssetLoader;
    type Error = std::io::Error;

    fn save<'a>(
        &'a self,
        writer: &'a mut Writer,
        asset: SavedAsset<'a, Self::Asset>,
        settings: &'a Self::Settings,
    ) -> BoxedFuture<'a, Result<TextSettings, Self::Error>> {
        Box::pin(async move {
            let text = format!("{}{}", asset.text.clone(), settings.appended);
            writer.write_all(text.as_bytes()).await?;
            Ok(TextSettings::default())
        })
    }
}