use std::path::PathBuf;

use crate::{graph::context::{CurrentContext, PathsToEntitiesIndex}, vault::CurrentVault, bevy_overlay_graph::ui::nodes::GraphStartingPositions};

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

        current_context.set_current_context(&vault_path, self.next.clone());

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
                current_context.set_current_context(&vault_path, previous);
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
        current_context.set_current_context(&vault_path, self.next.clone());
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

/// Action for expanding the context of a selected node 
#[derive(Clone)]
pub struct ExpandContextAction {
    previous: Option<PathBuf>,
    next: PathBuf,
}

impl Action for ExpandContextAction {
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


        let pe_index = world.get_resource::<PathsToEntitiesIndex>().unwrap();
        let entity = *pe_index.0.get(&self.next).unwrap();

        // We must remember to set the spawn position to the position of the node
        // that we are expanding from
        let pos = world.get::<bevy::transform::components::Transform>(entity).unwrap();
        let pos = pos.translation.truncate();
        let mut spawn = world.get_resource_mut::<GraphStartingPositions>().unwrap();
        spawn.set_pos(pos);

        world.send_event(crate::events::context::RequestContextExpand {
            target_path: self.next.clone(),
            target_entity: entity,
            as_visitors: true,
        });

        println!("Performing ExpandContextAction");
    }

    fn undo(&mut self, world: &mut bevy::prelude::World) {
        println!("Undoing ExpandContextAction");
    }

    fn redo(&mut self, world:  &mut bevy::prelude::World) {
        println!("Redoing ExpandContextAction");
    }
}

impl ExpandContextAction {
    pub fn new(next: PathBuf) -> Self {
        ExpandContextAction {
            previous: None,
            next,
        }
    }

}

/// Action for collapsing the context of a selected node
#[derive(Clone)]
pub struct CollapseContextAction {
    target: PathBuf,
    // collapsed: Vec<PathBuf>,
}

impl Action for CollapseContextAction {
    fn execute(&mut self, world: &mut bevy::prelude::World) {

        let pe_index = world.get_resource::<PathsToEntitiesIndex>().unwrap();
        let entity = *pe_index.0.get(&self.target).unwrap();

        world.send_event(crate::events::context::RequestContextCollapse {
            target_path: self.target.clone(),
            target_entity: entity,
        });

        println!("Performing CollapseContextAction");
    }

    fn undo(&mut self, world: &mut bevy::prelude::World) {
        println!("Undoing CollapseContextAction");
    }

    fn redo(&mut self, world:  &mut bevy::prelude::World) {
        println!("Redoing CollapseContextAction");
    }
}

impl CollapseContextAction {
    pub fn new(target: PathBuf) -> Self {
        CollapseContextAction {
            target,
        }
    }
}
