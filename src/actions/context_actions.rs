use std::path::PathBuf;

use crate::{graph::context::CurrentContext, vault::CurrentVault};

use super::Action;

#[derive(Clone)]
pub struct MoveToContextAction {
    previous: Option<PathBuf>,
    next: PathBuf,
}

impl Action for MoveToContextAction {
    fn execute(&mut self, world: &mut bevy::prelude::World) {
        
        let vault = world.get_resource::<CurrentVault>().unwrap();
        let vault_path: PathBuf = match &vault.vault {
            Some(vault) => vault.get_vault_path(),
            None => {
                println!("No vault set");
                return
            }
        };

        let mut current_context = world.get_resource_mut::<CurrentContext>().unwrap();

        match current_context.context.clone() {
            Some(cxt) => {
                self.previous = Some(cxt.get_path());
                self.next = self.next.clone();
            },
            None => {
                println!("No current context");
                self.previous = None;
                self.next = self.next.clone();
            }
        };

        current_context.set_current_context(vault_path, self.next.clone());

        println!("Performing MoveToContextAction");
    }

    fn undo(&mut self, world: &mut bevy::prelude::World) {
        let vault = world.get_resource::<CurrentVault>().unwrap();
        let vault_path: PathBuf = match &vault.vault {
            Some(vault) => vault.get_vault_path(),
            None => {
                println!("No vault set");
                return
            }
        };
        let mut current_context = world.get_resource_mut::<CurrentContext>().unwrap();
        match self.previous.clone() {
            Some(previous) => {
                current_context.set_current_context(vault_path, previous);
                println!("Undoing MoveToContextAction");
            },
            None => {
                println!("No previous context");
            }
        }
    }

    fn redo(&mut self, world:  &mut bevy::prelude::World) {
        let vault = world.get_resource::<CurrentVault>().unwrap();
        let vault_path: PathBuf = match &vault.vault {
            Some(vault) => vault.get_vault_path(),
            None => {
                println!("No vault set");
                return
            }
        };
        let mut current_context = world.get_resource_mut::<CurrentContext>().unwrap();
        current_context.set_current_context(vault_path, self.next.clone());
        println!("Redoing MoveToContextAction");
    }
}

impl MoveToContextAction {
    pub fn new(next: PathBuf) -> Self {
        MoveToContextAction {
            previous: None,
            next,
        }
    }

}