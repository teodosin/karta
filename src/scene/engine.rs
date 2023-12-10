// Main file for the active node graph evaluator
// on each context change, and on each edge connections change,
// the active node graph is updated

use bevy::ecs::world::World;

use crate::graph::{context::PathsToEntitiesIndex, nodes::GraphDataNode};

use super::scene::CurrentActive;



pub fn _evaluate_active_graph (
    world: &mut World,
){
    let mut data_nodes = world.query::<&GraphDataNode>();
    let active = world.get_resource::<CurrentActive>().unwrap().clone();
    let pe_index = world.get_resource::<PathsToEntitiesIndex>().unwrap().clone();

    // Get the root active node
    let root = match &active.active {
        Some(active) => active,
        None => {
            println!("No active context set");
            return
        }
    };

    // Get the actual entity
    let root_entity = match pe_index.0.get(root) {
        Some(root_entity) => root_entity,
        None => {
            println!("No root entity found for active context");
            return
        }
    };

    // // Check that the entity has a valid data return type
    let _root_entity = match data_nodes.get(world, *root_entity) {
        Ok(entity) => entity,
        Err(_) => {
            println!("Root entity is not a valid data node");
            return
        },
    };

    // RUN IT
    //root_entity.get_data(world);

}