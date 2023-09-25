
use bevy::prelude::*;

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
    edges: Query<&GraphEdge>,
    nodes: Query<&Transform, With<GraphNode>>,
    _cameras: Query<&Camera>,
    mut gizmos: Gizmos,
) {
    for edge in edges.iter() {
        let start = match nodes.get(edge.from) {
            Ok(node) => node,
            Err(_) => continue,
        };
        let end = match nodes.get(edge.to){
            Ok(node) => node,
            Err(_) => continue,
        };
        gizmos.line_2d(
            Vec2::new(start.translation.x, start.translation.y),
            Vec2::new(end.translation.x, end.translation.y),
            Color::GREEN,
        );
    }
}