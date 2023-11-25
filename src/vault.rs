// Handling the vault stuff
// Creating one if it doesn't exist

use std::{path::PathBuf, ffi::OsString};

use directories::ProjectDirs;

use bevy::{
    prelude::{
        Plugin, 
        App, 
        Startup, 
        Commands, 
        AssetApp, ResMut, Resource,
    }, 
    app::{PreStartup, Update, PreUpdate, PostUpdate}, ecs::schedule::{common_conditions::resource_changed, IntoSystemConfigs}
};
 

use crate::graph::context::CurrentContext;

use self::{context_asset::{ContextAsset, ContextAssetState, ContextAssetLoader, load_contexts}, asset_manager::{ImageLoadTracker, on_image_load}};

mod context_asset;
mod vault_asset;
mod asset_manager;

pub struct VaultPlugin;

impl Plugin for VaultPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(VaultOfVaults::new())
            .insert_resource(CurrentVault::new())

            .init_asset_loader::<ContextAssetLoader>()
            .init_resource::<ContextAssetState>()
            .init_asset::<ContextAsset>()

            .insert_resource(ImageLoadTracker::new())
            
            .add_systems(Startup, load_contexts)
            .add_systems(PreStartup, setup_vaults)
            // .add_systems(Update, use_assets)

            .add_systems(Update, on_vault_change.run_if(resource_changed::<CurrentVault>()))

            .add_systems(PostUpdate, on_image_load)
        ;

    }
}

#[derive(Resource, Debug, PartialEq)]
pub struct VaultOfVaults {
    pub vaults: Vec<KartaVault>,
}

impl VaultOfVaults {
    fn new() -> Self {
        let new_vault = VaultOfVaults {
            vaults: Vec::new(),
        };

        new_vault
    }

    pub fn add_vault(&mut self, vault: KartaVault) {
        self.vaults.push(vault);
    }

    pub fn remove_vault(&mut self, vault: KartaVault) {
        let index = self.vaults.iter().position(|v| *v == vault).unwrap();
        self.vaults.remove(index);
    }
}

fn setup_vaults(
    mut commands: Commands,
    mut vaults: ResMut<VaultOfVaults>,
    mut current_vault: ResMut<CurrentVault>,
){
    let project_dirs = ProjectDirs::from("com", "Teodosin", "Karta").unwrap();
    let config_dir = project_dirs.config_dir();

    let vault = KartaVault::new("/home/viktor/Pictures".to_string());
    current_vault.set_vault(vault.clone());
    vaults.add_vault(vault);
    
}

fn on_vault_change(
    mut commands: Commands,
    current_vault: ResMut<CurrentVault>,
    mut current_context: ResMut<CurrentContext>,
){
    let vault = current_vault.vault.clone().unwrap();
    let path = vault.get_root_path();
    println!("Changing current context to vault root: {:?}", path);
    current_context.set_current_context(path);
}

// RESOURCE STORING THE CURRENT TYPE
#[derive(Resource, Debug, PartialEq)]
pub struct CurrentVault {
    pub vault: Option<KartaVault>,
}

impl CurrentVault {
    fn new() -> Self {
        CurrentVault {
            vault: None,
        }
    }

    fn set_vault(&mut self, vault: KartaVault) {
        self.vault = Some(vault);
    }
}

// VAULT TYPE
#[derive(Debug, PartialEq, Clone)]
pub struct KartaVault{
    pub vault_folder_name: OsString,
    pub root: PathBuf,
}

impl KartaVault {
    fn new(path: String) -> Self {
        let new_vault = KartaVault {
            vault_folder_name: OsString::from(".kartaVault"),
            root: PathBuf::from(path),
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

