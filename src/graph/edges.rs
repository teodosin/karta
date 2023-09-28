
use bevy::{prelude::*, transform::commands};

use super::nodes::GraphNode;

// ----------------------------------------------------------------
// Component definitions

// A component for the data of an EDGE
#[derive(Component)]
pub struct GraphEdge {
    pub from: Entity,
    pub to: Entity,
    pub attributes: Vec<GraphEdgeAttribute>,
}

// A component for the attributes of an edge
#[derive(Component)]
pub struct GraphEdgeAttribute {
    pub name: String,
    pub value: f32,
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
            commands.entity(edge_entity).despawn_recursive();
        }
    }
}