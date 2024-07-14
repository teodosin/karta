use std::path::PathBuf;

use bevy::{app::{App, Plugin}, prelude::{Deref, Resource}};
use directories::ProjectDirs;
use fs_graph::graph::Graph;




pub struct VaultPlugin;

impl Plugin for VaultPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(VaultOfVaults::new())
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
    graph: Option<Graph>,
}

pub struct KartaVault;