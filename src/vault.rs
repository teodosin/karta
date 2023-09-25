// Handling the vault stuff
// Creating one if it doesn't exist

use bevy::prelude::{Resource, Plugin, App};

pub struct VaultPlugin;

impl Plugin for VaultPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(KartaVault::new())
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
        KartaVault {
            vault_folder_name: "kartaVault".to_string(),
            root: "home/viktor/Pictures".to_string(),
        }
    }

    pub fn get_root_path(&self) -> String {
        format!("/{}", self.root)
    }

    pub fn get_vault_path(&self) -> String {
        format!("/{}/{}", self.root, self.vault_folder_name)
    }
}