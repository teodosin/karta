//

use std::path::PathBuf;

use crate::{graph::edges::{GraphDataEdge, EdgeType}, bevy_overlay_graph::input::pointer::InputData};

use super::Action;

// Create edge
// Delete edge
#[derive(Clone)]
pub struct DeleteEdgeAction {
    source: PathBuf,
    target: PathBuf,
}

impl Action for DeleteEdgeAction {
    fn execute(&mut self, world: &mut bevy::ecs::world::World) {
        let input_data = world.get_resource::<InputData>().unwrap();

        let entity = input_data.latest_edge_entity.unwrap();

        // If the edge is a parent edge, it can't be deleted.
        let edge_type = world.get::<EdgeType>(entity).unwrap();
        if edge_type.etype == crate::graph::edges::EdgeTypes::Parent {
            return;
        }

        let edge_data = world.get::<GraphDataEdge>(entity).unwrap();

        self.source = edge_data.source.clone();
        self.target = edge_data.target.clone();

        world.despawn(entity);
    }

    fn undo(&mut self, graph: &mut bevy::ecs::world::World) {
        // graph.add_edge(&self.source, &self.target);
    }

    fn redo(&mut self, graph: &mut bevy::ecs::world::World) {
        // self.execute(graph);
    }
}

impl DeleteEdgeAction {
    pub fn new() -> Self {
        DeleteEdgeAction {
            source: PathBuf::from(""),
            target: PathBuf::from(""),
        }
    }
}
// Update edge
// Add attribute to edge
// Remove attribute from edge
// Update attribute on edge