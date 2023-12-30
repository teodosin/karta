//

use std::{ffi::{OsString, OsStr}, path::PathBuf};

use bevy::{prelude::{Entity, With, Vec2}, transform::components::Transform};
use bevy_mod_picking::selection::PickSelection;

use crate::{graph::{nodes::{GraphDataNode, ContextRoot, GraphNodeEdges, Pins}, context::{PathsToEntitiesIndex, CurrentContext}, node_types::{NodeTypes, type_to_data}}, input::pointer::InputData, ui::nodes::GraphViewNode, events::nodes::NodeSpawnedEvent, vault::{context_asset::{node_path_to_context_path, create_single_node_context}, CurrentVault}};

use super::Action;

// ------------------ CreateNodeAction ------------------

#[derive(Clone)]
pub struct CreateNodeAction {
    entity: Option<Entity>,
    ntype: NodeTypes,
    position: Vec2,
}



/// NOTE: The implementation here must be kept in sync with the implementation of spawn_node,
/// which is mostly called when expanding a node or changing context. 
impl Action for CreateNodeAction {
    fn execute(&mut self, world: &mut bevy::prelude::World) {

        let vault = world.get_resource::<CurrentVault>().unwrap();
        let vault_path = vault.vault.as_ref().unwrap().get_vault_path().clone();
        let context = world.get_resource::<CurrentContext>().unwrap();

        let cxt = match &context.context {
            None => {
                println!("No context set");
                return
            },
            Some(cxt) => cxt,
        };

        // Find the closest physical parent folder. The created node will be treated like it 
        // was created in this folder.
        let mut cpath = cxt.get_path();
        while !cpath.is_dir(){
            cpath.pop();
        }
        
        let name = OsString::from(self.ntype.to_string());
        let full_path = cpath.join(&name);
        
        let valid_path = get_valid_node_path(&vault_path, full_path.clone());

        println!("Creating node with path: {:?}", valid_path);
        
        // Could be worth it to map out the difference between spawn_node and this code. 
        let node_entity = world.spawn((
            GraphDataNode {
                path: valid_path.clone(),
                ntype: self.ntype,
                data: type_to_data(self.ntype)
            },
            GraphNodeEdges::default(),
        )).id();
        
        self.entity = Some(node_entity);

        let root_position = world.query_filtered::<&Transform, With<ContextRoot>>().single(world);
        
        world.send_event(NodeSpawnedEvent {
            entity: node_entity,
            path: valid_path.clone(),
            ntype: self.ntype,
            data: type_to_data(self.ntype),
            root_position: root_position.translation.truncate(),
            rel_target_position: Some(self.position - root_position.translation.truncate()),
            // rel_target_position: None,
            pinned_to_position: true,
        });
        
        // Update the PathsToEntitiesIndex
        let mut pe_index = world.get_resource_mut::<PathsToEntitiesIndex>().unwrap();
        pe_index.0.insert(valid_path.clone(), node_entity);

        // Create the context file
        create_single_node_context(&vault_path, self.ntype, &valid_path);
    }

    fn undo(&mut self, _world: &mut bevy::prelude::World) {
        println!("Undoing CreateNodeAction");
    }

    fn redo(&mut self, _world:  &mut bevy::prelude::World) {
        println!("Redoing CreateNodeAction");
    }
}

/// Function to validate proposed node path. Checks if the path exists as a physical file or a
/// respective context file exists. All node naming must pass through this function. Takes in the 
/// proposed node path and the whole vault path (name included) as arguments.
pub fn get_valid_node_path (
    vault_path: &PathBuf,
    node_path: PathBuf,
) -> PathBuf{
    
    let mut proposed_path = node_path.clone();
    let name: OsString = proposed_path.file_name().unwrap().into();
    let mut i = 1;
    loop {
        let context_exists = node_path_to_context_path(&vault_path, &proposed_path).exists();
        println!("Context exists: {}: {}", context_exists, node_path_to_context_path(&vault_path, &proposed_path).display());
        let physical_exists = proposed_path.exists();

        if !context_exists && !physical_exists {
            break
        }

        let new_name: OsString = format!("{}{}", name.to_str().unwrap(), i).into();

        proposed_path.set_file_name(new_name);

        println!("Proposed path: {:?}", proposed_path);

        i += 1;

        assert!(node_path.extension() == proposed_path.extension())
    }

    proposed_path
}

impl CreateNodeAction {
    pub fn new(
        // entity: Entity,
        ntype: NodeTypes, 
        pos: Vec2
    ) -> Self {
        CreateNodeAction {
            entity: None,
            ntype, 
            position: pos,
        }
    }
}

// ------------------ DeleteNodeAction ------------------

// ------------------ RemoveNodeAction ------------------
// There is a distinction to be made between removing and deleting. Removing a node
// just removes it from the current context. If the node doesn't exist in any other context,
// it will also be deleted after a warning. If the current context is the parent context,
// but it exists in other contexts, upon removal the user should also probably be 
// notified if the node has to be moved to a different host context.

// ------------------ EditNodeAction ------------------

// ------------------ PinToPositionAction ------------------
#[derive(Clone)]
pub struct PinToPositionAction {
    pins: Option<Vec<(Entity, bool)>>,
}

impl Action for PinToPositionAction {
    fn execute(&mut self, world: &mut bevy::prelude::World) {
        // let mut selected = world.query_filtered::<Entity, (With<GraphViewNode>, With<Selected>)>();
        let mut selected = world.query::<(Entity, &mut Pins, &PickSelection)>();

        let clicked_node = self.get_latest_clicked_node(world);
        
        let mut targets: Vec<Entity> = Vec::new();

        match clicked_node {
            None => return,
            Some(node) => {
                targets.push(node);
            }
        }

        
        println!("Size of selection when pinning: {}", selected.iter(world).filter(| (_, _, pick) | pick.is_selected).count());
        for (node, pin, pick) in selected.iter(world) {
            if !pick.is_selected { continue }

            // If the node is already pinned, we don't want to pin it again
            if !pin.position {
                targets.push(node);
                println!("Not already pinned");
                match self.pins {
                    Some(ref mut pins) => pins.push((node, false)),
                    None => self.pins = Some(vec![(node, false)]),
                }
                
            }
        }

        for target in targets {
            selected.get_mut(world, target).unwrap().1.position = true;
        }
        
        println!("Performing PinToPositionAction");
    }

    fn undo(&mut self, world: &mut bevy::prelude::World) {
        println!("Undoing PinToPositionAction");
    }

    fn redo(&mut self, world:  &mut bevy::prelude::World) {
        println!("Redoing PinToPositionAction");
    }
}

impl PinToPositionAction {
    pub fn new() -> Self {
        PinToPositionAction {
            pins: None,
        }
    }

    fn get_latest_clicked_node(
        &self, world: &mut bevy::prelude::World
    ) -> Option<Entity> {
        let input_data = world.get_resource::<InputData>().unwrap();
        let path = input_data.latest_click_nodepath.clone().unwrap();
        let index = world.get_resource::<PathsToEntitiesIndex>().unwrap();
        let node = index.0.get(&path);
        Some(*node.unwrap())
    }
}

// ------------------ UnpinToPositionAction ------------------
#[derive(Clone)]
pub struct UnpinToPositionAction {
    pins: Option<Vec<(Entity, bool)>>,
}

impl Action for UnpinToPositionAction {
    fn execute(&mut self, world: &mut bevy::prelude::World) {
            
            let mut selected = world.query::<(Entity, &mut Pins, &PickSelection)>();
    
            let clicked_node = self.get_latest_clicked_node(world);
            
            let mut targets: Vec<Entity> = Vec::new();
    
            match clicked_node {
                None => return,
                Some(node) => {
                    targets.push(node);
                }
            }
    
            
            println!("Size of selection when unpinning: {}", selected.iter(world).filter(| (_, _, pick) | pick.is_selected).count());
            for (node, pin, pick) in selected.iter(world) {
                if !pick.is_selected { continue }
    
                if pin.position {
                    targets.push(node);
                    match self.pins {
                        Some(ref mut pins) => pins.push((node, true)),
                        None => self.pins = Some(vec![(node, true)]),
                    }
                }
            }

            for target in targets {
                println!("Removing pin");
                selected.get_mut(world, target).unwrap().1.position = false;
            }
    }

    fn undo(&mut self, world: &mut bevy::prelude::World) {
        println!("Undoing UnpinToPositionAction");
    }

    fn redo(&mut self, world:  &mut bevy::prelude::World) {
        println!("Redoing UnpinToPositionAction");
    }

}

impl UnpinToPositionAction {
    pub fn new() -> Self {
        UnpinToPositionAction {
            pins: None,
        }
    }

    fn get_latest_clicked_node(
        &self, world: &mut bevy::prelude::World
    ) -> Option<Entity> {
        let input_data = world.get_resource::<InputData>().unwrap();
        let path = input_data.latest_click_nodepath.clone().unwrap();
        let index = world.get_resource::<PathsToEntitiesIndex>().unwrap();
        let node = index.0.get(&path);
        Some(*node.unwrap())
    }
}


// ------------------ PinToPresenceAction ------------------

// ------------------ UnpinToPresenceAction ------------------