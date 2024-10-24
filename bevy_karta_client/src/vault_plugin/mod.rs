use std::path::PathBuf;

use std::fs::{self, File};
use std::io::{self, BufRead, Write};

use bevy::prelude::*;
use directories::ProjectDirs;
use karta_server::prelude::*;
use native_dialog::FileDialog;

use crate::prelude::{CurrentContext, KartaContext};

pub struct VaultPlugin;

impl Plugin for VaultPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(VaultOfVaults::new())
            .insert_resource(CurrentVault::new())
            .add_systems(PreStartup, load_vaults)
            .add_systems(PostUpdate, set_context_on_vault_change.run_if(resource_changed::<CurrentVault>));
    }
}

fn load_vaults(mut vaults: ResMut<VaultOfVaults>) {
    if let Err(err) = vaults.load_vaults_from_config() {
        error!("Failed to load vaults from config: {}", err);
    }
}

/// Resource that stores all of a user's vaults.
#[derive(Resource)]
pub struct VaultOfVaults {
    pub search_input: String,
    project_dir: ProjectDirs,
    vaults: Vec<KartaVault>,
}

impl VaultOfVaults {
    fn new() -> Self {
        VaultOfVaults {
            search_input: String::new(),
            project_dir: ProjectDirs::from("com", "Teodosin", "Karta").unwrap(),
            vaults: vec![],
        }
    }

    pub fn config_dir(&self) -> PathBuf {
        self.project_dir.config_dir().to_path_buf()
    }

    pub fn create_vault(&mut self, cur: &mut CurrentVault) -> io::Result<()> {
        let search_path = PathBuf::from(self.search_input.clone());

        if self.vaults.contains(&KartaVault::new(search_path.clone())) {
            return Ok(());
        }

        if search_path.exists() && search_path.is_dir() {
            let vault = KartaVault::new(search_path);
            self.vaults.push(vault);
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
                    cur.set_vault(vault);
                }
                None => {
                    println!("No folder selected");
                    return Ok(())
                }
            }
        }
        self.save_vaults_to_config()?;
        Ok(())
    }

    fn config_file_path(&self) -> PathBuf {
        self.config_dir().join("vaults.txt")
    }

    pub fn save_vaults_to_config(&self) -> io::Result<()> {
        fs::create_dir_all(self.config_dir())?;
        let mut file = File::create(self.config_file_path())?;

        for vault in &self.vaults {
            writeln!(file, "{}", vault.path.display())?;
        }
        Ok(())
    }

    pub fn load_vaults_from_config(&mut self) -> io::Result<()> {
        let path = self.config_file_path();
        if !path.exists() {
            return Ok(());
        }

        let file = File::open(path)?;
        let reader = io::BufReader::new(file);

        self.vaults.clear();
        for line in reader.lines() {
            let path = PathBuf::from(line?);
            if path.exists() && path.is_dir() {
                self.vaults.push(KartaVault::new(path));
            }
        }
        Ok(())
    }

    pub fn get_vaults(&self) -> &Vec<KartaVault> {
        &self.vaults
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

    pub fn set_vault(&mut self, vault: KartaVault) {
        println!("Setting vault to be: {:?}", vault.path);
        let name = vault.path.file_stem();
        let name: &str = match name {
            Some(name) => name.to_str().unwrap(),
            None => return,
        };
        self.graph = Some(GraphCommands::new(
            name,
            vault.path.clone(),
            Some(vault.path.clone()),
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
