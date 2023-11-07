//

use bevy::prelude::{Entity, With};

use crate::{graph::{nodes::PinnedToPosition, context::{PathsToEntitiesIndex, Selected}}, input::pointer::InputData, ui::nodes::GraphViewNode};

use super::Action;

// ------------------ CreateNodeAction ------------------

#[derive(Clone)]
pub struct CreateNodeAction {
    node: String,
}

impl Action for CreateNodeAction {
    fn execute(&mut self, world: &mut bevy::prelude::World) {
        println!("Performing CreateNodeAction");
    }

    fn undo(&mut self, world: &mut bevy::prelude::World) {
        println!("Undoing CreateNodeAction");
    }

    fn redo(&mut self, world:  &mut bevy::prelude::World) {
        println!("Redoing CreateNodeAction");
    }
}

impl Default for CreateNodeAction {
    fn default() -> Self {
        CreateNodeAction {
            node: String::from("Default Node"),
        }
    }
}

impl CreateNodeAction {
    pub fn new(node: String) -> Self {
        CreateNodeAction {
            node,
        }
    }
}

// ------------------ DeleteNodeAction ------------------

// ------------------ EditNodeAction ------------------

// ------------------ PinToPositionAction ------------------
#[derive(Clone)]
pub struct PinToPositionAction {
    pins: Option<Vec<(Entity, bool)>>,
}

impl Action for PinToPositionAction {
    fn execute(&mut self, world: &mut bevy::prelude::World) {
        let mut selected = world.query_filtered::<Entity, (With<GraphViewNode>, With<Selected>)>();
        let mut pinned = world.query_filtered::<Entity, (With<GraphViewNode>, With<PinnedToPosition>)>();

        let mut targets: Vec<Entity> = Vec::new();
        
        for node in selected.iter(world) {
            println!("Looping through selected");
            match pinned.get(world, node){
                Ok(_) => {
                    println!("It's a match in pinven");
                    match self.pins {
                        Some(ref mut pins) => pins.push((node, true)),
                        None => self.pins = Some(vec![(node, true)]),
                    }
                }
                Err(_) => {
                    targets.push(node);
                    match self.pins {
                        Some(ref mut pins) => pins.push((node, false)),
                        None => self.pins = Some(vec![(node, false)]),
                    }
                }
            }
        }
        for target in targets {
            println!("Adding pin component");
            world.entity_mut(target).insert(PinnedToPosition);
        }
        
        println!("Performing PinToPositionAction");
    }

    fn undo(&mut self, world: &mut bevy::prelude::World) {
        let node = self.get_latest_clicked_node(world).unwrap();
        world.entity_mut(node).remove::<PinnedToPosition>();
        println!("Undoing PinToPositionAction");
    }

    fn redo(&mut self, world:  &mut bevy::prelude::World) {
        let node = self.get_latest_clicked_node(world).unwrap();
        world.entity_mut(node).insert(PinnedToPosition);
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
        let path = input_data.latest_click_entity.clone().unwrap();
        let index = world.get_resource::<PathsToEntitiesIndex>().unwrap();
        let node = index.0.get(&path);
        Some(*node.unwrap())
    }
}

// ------------------ UnpinToPositionAction ------------------
#[derive(Clone)]
pub struct UnpinToPositionAction {
    node: Entity,
}

impl Action for UnpinToPositionAction {
    fn execute(&mut self, world: &mut bevy::prelude::World) {
        world.entity_mut(self.node).remove::<PinnedToPosition>();
        println!("Performing UnpinToPositionAction");
    }

    fn undo(&mut self, world: &mut bevy::prelude::World) {
        world.entity_mut(self.node).insert(PinnedToPosition);
        println!("Undoing UnpinToPositionAction");
    }

    fn redo(&mut self, world:  &mut bevy::prelude::World) {
        world.entity_mut(self.node).remove::<PinnedToPosition>();
        println!("Redoing UnpinToPositionAction");
    }

}

impl UnpinToPositionAction {
    pub fn new(node: Entity) -> Self {
        UnpinToPositionAction {
            node,
        }
    }
}


// ------------------ PinToPresenceAction ------------------

// ------------------ UnpinToPresenceAction ------------------