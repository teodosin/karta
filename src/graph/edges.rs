
use bevy::prelude::*;

use super::{nodes::GraphNode, context::CurrentContext};

pub struct EdgesPlugin;

impl Plugin for EdgesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(PostUpdate, draw_edges)

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



// ----------------------------------------------------------------
// Crude drawing of edges
pub fn draw_edges(
    mut commands: Commands,
    edges: Query<(Entity, &GraphEdge)>,
    nodes: Query<&Transform, With<GraphNode>>,
    mut gizmos: Gizmos,
) {
    for (edge_entity, edge_data) in edges.iter() {
        let start = match nodes.get(edge_data.from) {
            Ok(node) => node,
            Err(_) => {
                //commands.entity(edge_entity).despawn_recursive();
                continue
            },
        };
        let end = match nodes.get(edge_data.to){
            Ok(node) => node,
            Err(_) => {
                //commands.entity(edge_entity).despawn_recursive();
                continue
            },
        };
        gizmos.line_2d(
            Vec2::new(start.translation.x, start.translation.y),
            Vec2::new(end.translation.x, end.translation.y),
            Color::GREEN,
        );
    }
}

pub fn despawn_edges(
    mut commands: Commands,
    mut edges: Query<(Entity, &GraphEdge)>,
    nodes: Query<&GraphNode>,
) {
    for (edge_entity, edge_data) in edges.iter_mut() {
        if nodes.get(edge_data.from).is_err() || nodes.get(edge_data.to).is_err() {
            println!("Despawning edge");
            commands.entity(edge_entity).despawn_recursive();
        }
    }
}