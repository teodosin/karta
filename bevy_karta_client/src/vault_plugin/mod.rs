use std::path::PathBuf;

use directories::ProjectDirs;
use bevy::prelude::*;
use fs_graph::prelude::*;
use native_dialog::FileDialog;




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

    pub fn set_vault(&mut self, vault: KartaVault){
        println!("Setting vault to be: {:?}", vault.path);
        let name = vault.path.file_stem();
        let name: &str = match name {
            Some(name) => name.to_str().unwrap(),
            None => return,
        };
        self.graph = Some(GraphAgdb::new(name, vault.path.clone(), None));
        self.vault = Some(vault);
    }
}

#[derive(Clone)]
pub struct KartaVault {
    path: PathBuf
}

impl KartaVault {
    pub fn new(path: PathBuf) -> Self {
        KartaVault {
            path,
        }
    }
}

fn initialise_default_vault_until_theres_a_vault_menu(
    mut vaults: ResMut<VaultOfVaults>,
    mut current_vault: ResMut<CurrentVault>,

    _: NonSend<bevy::winit::WinitWindows>,
) {
    let folder = FileDialog::new()
        .set_title("Select Karta Vault location")
        .show_open_single_dir();

    let folder = match folder {
        Ok(folder) => folder,
        Err(_) => None,
    };

    match folder {
        Some(folder) => {
            if !folder.is_dir() {
                println!("Not a folder");
                return;
            }

            let vault = KartaVault::new(folder);
            vaults.add_vault(vault.clone());
            current_vault.set_vault(vault)
        },
        None => {
            println!("No folder selected");
        }
    }
}