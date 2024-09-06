use std::path::PathBuf;

use directories::ProjectDirs;
use bevy::prelude::*;
use fs_graph::prelude::*;
use native_dialog::FileDialog;

use crate::prelude::{CurrentContext, KartaContext};




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
    pub vault: Option<KartaVault>,
    pub graph: Option<GraphCommands>,
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
        self.graph = Some(GraphCommands::new(name, vault.path.clone(), None));
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
    mut current_context: ResMut<CurrentContext>,

    _: NonSend<bevy::winit::WinitWindows>,
) {
    print!("Enter vault path (leave empty for file dialog): ");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("Failed to read line");
    let input = input.trim();

    let folder = if input.is_empty() {
        FileDialog::new()
            .set_title("Select Karta Vault location")
            .show_open_single_dir()
            .unwrap_or(None)
    } else {
        let path = std::path::PathBuf::from(input);
        if !path.is_dir() {
            panic!("The provided path is not a valid directory");
        }
        Some(path)
    };


    match folder {
        Some(folder) => {
            if !folder.is_dir() {
                println!("Not a folder");
                return;
            }

            let vault = KartaVault::new(folder);
            vaults.add_vault(vault.clone());
            current_vault.set_vault(vault);
            current_context.set(NodePath::root());
        },
        None => {
            println!("No folder selected");
            return;
        }
    }
}