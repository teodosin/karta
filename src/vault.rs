// Handling the vault stuff
// Creating one if it doesn't exist

use std::{path::PathBuf, ffi::OsString};

use bevy_file_dialog::{FileDialogPlugin, FileDialog};
use directories::ProjectDirs;

use bevy::{
    prelude::{
        Plugin, 
        App, 
        Startup, 
        AssetApp, ResMut, Resource,
    }, 
    app::{PreStartup, Update, PreUpdate}, ecs::{schedule::{common_conditions::resource_changed, IntoSystemConfigs}, event::EventWriter}
};
 

use crate::{graph::context::CurrentContext, vault::vault_asset::{VAULTS_FILE_NAME, VaultAsset}, ui::vault_menu::SpawnVaultMenu};

use self::{context_asset::{ContextAsset, ContextAssetState, ContextAssetLoader, load_contexts}, asset_manager::{ImageLoadTracker, on_image_load}};

mod context_asset;
mod vault_asset;
mod asset_manager;

pub struct VaultPlugin;

impl Plugin for VaultPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(FileDialogPlugin)
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

            .add_systems(PreUpdate, on_image_load)
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

    pub fn _remove_vault(&mut self, vault: KartaVault) {
        let index = self.vaults.iter().position(|v| *v == vault).unwrap();
        self.vaults.remove(index);
    }
}

fn setup_vaults(
    mut vaults: ResMut<VaultOfVaults>,
    mut current_vault: ResMut<CurrentVault>,
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

    // Check if the file exists
    if full_path.exists() {
        println!("Vaults file exists");
        // Load the file
        let vaults_file = std::fs::read_to_string(full_path).expect("Could not read vaults file");
        println!("Vaults file: {:?}", vaults_file);
        // Deserialize the file
        let vaultassets: VaultAsset = ron::de::from_str(&vaults_file).expect("Could not deserialize vaults file");
        println!("Vaults: {:?}", vaults);
        // Add the vaults to the vaults resource
        for vault in &vaultassets.vaults {
            let vault = KartaVault::new(vault.vault_root_path.clone().into());
            vaults.add_vault(vault);
        }
    }
    else {
        println!("Vaults file does not exist");
        // Create the file
        //std::fs::File::create(full_path).expect("Could not create vaults file");
    }

    let vault = KartaVault::new(PathBuf::from("/home/viktor/Pictures"));
    current_vault.set_vault(vault.clone());
    vaults.add_vault(vault);
    
}

fn on_vault_change(
    current_vault: ResMut<CurrentVault>,
    mut current_context: ResMut<CurrentContext>,
    mut dialog: ResMut<FileDialog>,
    mut event: EventWriter<SpawnVaultMenu>,
){
    let vault = match &current_vault.vault {
        Some(vault) => {
            println!("Vault changed to: {:?}", vault);
            vault
        },
        None => {
            println!("Vault changed to: None");
            
            // let mut vault = dialog.save_file(b"hello".to_vec());

            // Spawn vault modal dialog

            event.send(SpawnVaultMenu);

            return
        }
    };
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
    fn new(path: PathBuf) -> Self {
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

