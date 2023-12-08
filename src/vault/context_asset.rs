//

use std::{path::PathBuf, fs::DirEntry};

use bevy::{prelude::Vec2, ecs::system::{Resource, Commands, Res, ResMut}, asset::{AssetServer, Assets, Handle, Asset, AssetLoader, io::Reader, LoadContext, AsyncReadExt}, reflect::TypePath, utils::BoxedFuture};
use serde::Deserialize;
use thiserror::Error;

use super::CurrentVault;

pub fn load_contexts(
    mut _commands: Commands,
    asset_server: Res<AssetServer>,
    _context_asset_state: Res<ContextAssetState>,
    assets: ResMut<Assets<ContextAsset>>,
    vault: Res<CurrentVault>,
){
    let file_name = "thingy.context";

    let vault = match &vault.vault {
        Some(vault) => vault,
        None => {
            println!("No vault set");
            return
        }
    };
    
    let path: PathBuf = vault.get_vault_path().join(file_name);
    let context_assets: Handle<ContextAsset> = 
        asset_server.load(path);
    println!("context_assets: {:?}", context_assets);

    let _data = assets.get(&context_assets);
    // commands.insert_resource(ContextAssets {
    //     handle: context_assets,
    // });
}

#[derive(Resource, Default)]
pub struct ContextAssetState {
    pub _handle: Handle<ContextAsset>,
}

#[derive(Asset, Debug, Deserialize, TypePath, Default)]
pub struct ContextAsset {
    pub self_path: String,

    #[serde(default = "Vec::new")]
    pub edges: Vec<EdgeSerial>,

    #[serde(default = "Vec::new")]
    pub nodes: Vec<NodeSerial>,
}

#[derive(Debug, Deserialize, Default)]
pub struct EdgeSerial {
    pub path: String,

    #[serde(default = "Vec2::default")]
    pub relative_position: Vec2,
}
 
#[derive(Debug, Deserialize, Default)]
pub struct NodeSerial {
    pub name: String,

    #[serde(default = "Vec2::default")]
    pub relative_position: Vec2,

    #[serde(default = "Vec::new")]
    pub edges: Vec<String>,
}



#[derive(Default)]
pub struct ContextAssetLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ContextAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could load shader: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

impl AssetLoader for ContextAssetLoader {
    type Asset = ContextAsset;
    type Settings = ();
    type Error = ContextAssetLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let asset: ContextAsset = ron::de::from_bytes::<ContextAsset>(&bytes)?;
            Ok(asset)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["context"]
    }

}

pub fn _dir_or_file_is_hidden(
    entry: &DirEntry,
) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| !s.starts_with("."))
        .unwrap_or(false)
}