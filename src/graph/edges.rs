
use bevy::prelude::*;

use super::{nodes::GraphNode, context::CurrentContext};

pub struct EdgesPlugin;

impl Plugin for EdgesPlugin {
    fn build(&self, app: &mut App) {
        app

            .add_systems(Last, despawn_edges
                .run_if(resource_changed::<CurrentContext>())
            )
        ;
    }
}

// ----------------------------------------------------------------
// Component definitions

// A component for the most basic data of an EDGE
#[derive(Component, Reflect)]
pub struct GraphEdge {
    pub from: Entity,
    pub to: Entity,
}

pub fn create_edge(from: &Entity, to: &Entity, commands: &mut Commands){
    println!("Creating edge from {:?} to {:?}", from, to);
    commands.spawn((GraphEdge {
        from: *from,
        to: *to,
    },));
}

pub fn despawn_edges(
    mut commands: Commands,
    mut edges: Query<(Entity, &GraphEdge)>,
    nodes: Query<&GraphNode>,
) {
    for (edge_entity, edge_data) in edges.iter_mut() {
        if nodes.get(edge_data.from).is_err() || nodes.get(edge_data.to).is_err() {
            println!("Despawning edge");
            //commands.entity(edge_entity).despawn_recursive();
        }
    }
}
