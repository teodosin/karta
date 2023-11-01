// Drawing the edges

use bevy::prelude::*;

use crate::graph::{edges::GraphEdge, nodes::GraphNode};

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
                commands.entity(edge_entity).despawn_recursive();
                continue
            },
        };
        let end = match nodes.get(edge_data.to){
            Ok(node) => node,
            Err(_) => {
                commands.entity(edge_entity).despawn_recursive();
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