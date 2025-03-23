use std::path::PathBuf;

use std::fs::{self, File};
use std::io::{self, BufRead, Write};

use bevy::prelude::*;
use directories::ProjectDirs;
use karta_server::prelude::karta_service::KartaService;
use karta_server::prelude::*;
use native_dialog::FileDialog;

use crate::prelude::{CurrentContext, KartaContext};

pub struct VaultPlugin;

impl Plugin for VaultPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(VaultOfVaults::new())
            .insert_resource(CurrentVault::new())
            .add_systems(PostUpdate, set_context_on_vault_change.run_if(resource_changed::<CurrentVault>));
    }
}

/// Resource that stores all of a user's vaults.
#[derive(Resource)]
pub struct VaultOfVaults {
    pub search_input: String,
    vaults: Vaults,
}

impl VaultOfVaults {
    fn new() -> Self {
        VaultOfVaults {
            search_input: String::new(),
            vaults: get_vaults_config(),
        }
    }


    pub fn create_vault(&mut self, cur: &mut CurrentVault) -> io::Result<()> {
        let search_path = PathBuf::from(self.search_input.clone());

        if self.vaults.get().contains(&search_path) {
            return Ok(());
        }

        if search_path.exists() && search_path.is_dir() {
            let vault_path = search_path;
            self.vaults.add_vault(&vault_path);
            
        } else {
            let folder = FileDialog::new()
                .set_title("Select Karta Vault location")
                .show_open_single_dir()
                .unwrap_or(None);

            match folder {
                Some(folder) => {
                    if !folder.is_dir() {
                        println!("Not a folder");
                        return Ok(());
                    }

                    let vault = KartaVault::new(folder);
                    self.vaults.add_vault(&vault.path);
                    cur.set_vault(vault);
                }
                None => {
                    println!("No folder selected");
                    return Ok(())
                }
            }
        }
        self.vaults.save();
        Ok(())
    }

    pub fn config(&self) -> &Vaults {
        &self.vaults
    }
}

/// Resource that stores the current vault.
#[derive(Resource)]
pub struct CurrentVault {
    pub vault: Option<KartaVault>,
    pub graph: Option<KartaService>,
}

impl CurrentVault {
    pub fn new() -> Self {
        CurrentVault {
            vault: None,
            graph: None,
        }
    }

    pub fn set_vault(&mut self, vault: KartaVault) {
        println!("Setting vault to be: {:?}", vault.path);
        let name = vault.path.file_stem();
        let name: &str = match name {
            Some(name) => name.to_str().unwrap(),
            None => return,
        };
        self.graph = Some(KartaService::new(
            name,
            vault.path.clone(),
            vault.path.clone(),
        ));
        self.vault = Some(vault);
    }
}

fn set_context_on_vault_change(
    mut ctx: ResMut<CurrentContext>,
    vault: Res<CurrentVault>,
){
    ctx.set(NodePath::root());
}

#[derive(Clone, PartialEq)]
pub struct KartaVault {
    path: PathBuf,
}

impl KartaVault {
    pub fn new(path: PathBuf) -> Self {
        KartaVault { path }
    }
}
