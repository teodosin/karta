use std::path::PathBuf;

use directories::ProjectDirs;
use bevy::prelude::*;
use fs_graph::prelude::*;




pub struct VaultPlugin;

impl Plugin for VaultPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(VaultOfVaults::new())
            .insert_resource(CurrentVault::new())

            .add_systems(PreStartup, initialise_default_vault_until_theres_a_vault_menu)
        ;
    }
}

/// Resource that stores all of a user's vaults. 
#[derive(Resource)]
pub struct VaultOfVaults {
    project_dir: ProjectDirs,
    vaults: Vec<KartaVault>,
}

impl VaultOfVaults {
    fn new() -> Self {
        VaultOfVaults {
            project_dir: ProjectDirs::from("com", "Teodosin", "Karta").unwrap(),
            vaults: vec![],
        }
    }

    pub fn config_dir(&self) -> PathBuf {
        self.project_dir.config_dir().to_path_buf()
    }

    pub fn add_vault(&mut self, vault: KartaVault) {
        self.vaults.push(vault);
    }
}

/// Resource that stores the current vault. 
#[derive(Resource)]
pub struct CurrentVault {
    vault: Option<KartaVault>,
    graph: Option<GraphAgdb>,
}

impl CurrentVault {
    pub fn new() -> Self {
        CurrentVault {
            vault: None,
            graph: None,
        }
    }
}

pub struct KartaVault;

fn initialise_default_vault_until_theres_a_vault_menu(
    vault_of_vaults: Res<VaultOfVaults>,
    current_cault: Res<CurrentVault>,
) {

}