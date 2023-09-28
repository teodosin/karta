// Handling the vault stuff
// Creating one if it doesn't exist

use std::fs::DirEntry;

use bevy::{
    prelude::{
        Plugin, 
        App, 
        Vec2, 
        AddAsset, 
        Res, 
        AssetServer, 
        Startup, 
        Handle, 
        Commands, 
        Assets, 
        PostStartup, Resource, info, Update
    }, 
    asset::{
        AssetLoader, LoadedAsset, LoadContext
    }, reflect::{TypeUuid, TypePath}, utils::BoxedFuture
};
use serde::Deserialize;

pub struct VaultPlugin;

impl Plugin for VaultPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(KartaVault::new())
            .add_asset::<ContextAsset>()
            .init_asset_loader::<ContextAssetLoader>()

            .add_systems(Startup, load_assets)
            .add_systems(Update, use_assets)
        ;

    }
}

#[derive(Resource, Debug)]
pub struct KartaVault{
    pub vault_folder_name: String,
    pub root: String,
}

impl KartaVault {
    fn new() -> Self {
        let new_vault = KartaVault {
            vault_folder_name: ".kartaVault".to_string(),
            root: "home/viktor/Pictures".to_string(),
        };

        // Check if the folder exists in the root. If not, create it.
        // If it does, do nothing.
        let path = format!("/{}/{}", new_vault.root, new_vault.vault_folder_name);

        if !std::path::Path::new(&path).exists() {
            std::fs::create_dir(&path).expect("Could not create vault folder");
        }

        new_vault
    }

    pub fn get_root_path(&self) -> String {
        format!("/{}", self.root)
    }

    pub fn get_vault_path(&self) -> String {
        format!("/{}/{}", self.root, self.vault_folder_name)
    }
}

fn use_assets(
    ron_assets: Res<Assets<ContextAsset>>,
    data_assets: Res<ContextAssets>,
){
    let data_str = ron_assets.get(&data_assets.handle);
    match data_str {
        Some(data) => {
            info!("data: {:?}", data);
        },
        None => {
            info!("data: None");
        }
    }
}

fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
){
    let context_assets: Handle<ContextAsset> = asset_server.load("/home/viktor/Pictures/.kartaVault/thingy.context");
    println!("context_assets: {:?}", context_assets);

    commands.insert_resource(ContextAssets {
        handle: context_assets,
    });
}

#[derive(Resource, Debug)]
struct ContextAssets {
    pub handle: Handle<ContextAsset>,
}

#[derive(Debug, Deserialize, TypeUuid, TypePath, Default)]
#[uuid = "d0b0c5a0-0b0b-4c0b-8b0b-0b0b0b0b0b0b"]
pub struct ContextAsset {
    pub self_path: String,
    pub edges: Vec<EdgeSerial>,
    pub nodes: Vec<NodeSerial>,
    //pub edges: Vec<EdgeSerial>,
}

#[derive(Debug, Deserialize, Default)]
pub struct EdgeSerial {
    pub path: String,
    pub relative_position: Vec2,
}
 
#[derive(Debug, Deserialize, Default)]
pub struct NodeSerial {
    pub name: String,
    pub relative_position: Vec2,
    pub edges: Vec<String>,
}

#[derive(Default)]
pub struct ContextAssetLoader;

impl AssetLoader for ContextAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let data_str = ron::de::from_bytes::<ContextAsset>(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(data_str));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["context"]
    }
}

pub fn dir_or_file_is_hidden(
    entry: &DirEntry,
) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| !s.starts_with("."))
        .unwrap_or(false)
}