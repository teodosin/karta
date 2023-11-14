// Handling the vault stuff
// Creating one if it doesn't exist

use std::{fs::DirEntry, path::PathBuf, ffi::OsString};

use bevy::{
    prelude::{
        Plugin, 
        App, 
        Vec2, 
        Res, 
        AssetServer, 
        Startup, 
        Handle, 
        Commands, 
        Assets, 
        AssetApp, ResMut, Resource, Asset
    }, 
    asset::{
        AssetLoader, LoadContext, io::Reader, AsyncReadExt
    }, utils::BoxedFuture, reflect::TypePath
};
// use thiserror_core::display::DisplayAsDisplay;
// use futures_lite::AsyncReadExt;
use serde::Deserialize;
use thiserror::Error;

pub struct VaultPlugin;

impl Plugin for VaultPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(KartaVault::new())
            // .register_asset_loader(ContextAssetLoader)
            .init_asset_loader::<ContextAssetLoader>()
            .init_resource::<ContextAssetState>()
            .init_asset::<ContextAsset>()

            .add_systems(Startup, load_assets)
            // .add_systems(Update, use_assets)
        ;

    }
}

#[derive(Resource, Debug)]
pub struct KartaVault{
    pub vault_folder_name: OsString,
    pub root: PathBuf,
}

impl KartaVault {
    fn new() -> Self {
        let new_vault = KartaVault {
            vault_folder_name: OsString::from(".kartaVault"),
            root: PathBuf::from("/home/viktor"),
        };

        let path: PathBuf = new_vault.root.join(&new_vault.vault_folder_name);

        println!("Vault path: {:?}", path);

        if path.exists() {
            println!("Vault folder already exists");
            return new_vault
        }
        else {
            println!("Vault folder does not exist");
            std::fs::create_dir(path).expect("Could not create vault folder");
        }


        new_vault
    }

    pub fn get_root_path(&self) -> PathBuf {
        self.root.clone()
    }

    pub fn get_vault_path(&self) -> PathBuf {
        let vault_path = self.root.join(&self.vault_folder_name);
        vault_path
    }
}

// fn use_assets(
//     ron_assets: Res<Assets<ContextAsset>>,
//     data_assets: Res<ContextAssets>,
// ){
//     let data_str = ron_assets.get(&data_assets.handle);
//     match data_str {
//         Some(data) => {
//             //info!("data: {:?}", data);
//         },
//         None => {

//         }
//     }
// }

fn load_assets(
    mut _commands: Commands,
    asset_server: Res<AssetServer>,
    _context_asset_state: Res<ContextAssetState>,
    assets: ResMut<Assets<ContextAsset>>,
    vault: Res<KartaVault>,
){
    let file_name = "thingy.context";
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
struct ContextAssetState {
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