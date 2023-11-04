// Drawing the edges

use bevy::prelude::*;
use bevy_prototype_lyon::{shapes, prelude::{Fill, ShapeBundle, GeometryBuilder, Path, Stroke}};

use crate::graph::{edges::GraphEdge, nodes::GraphNode, graph_cam::ViewData};

pub fn add_edge_ui(
    commands: &mut Commands,
    edge_entity: Entity,
    view_data: &mut ViewData,
){
    let line = shapes::Line(
        Vec2::ZERO, Vec2::ZERO
    );

    commands.entity(edge_entity).insert((
        ShapeBundle {
            path: GeometryBuilder::build_as(&line),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, view_data.bottom_z),
                ..default()
            },
            ..default()
        },
        Fill::color(Color::ORANGE_RED),
        Stroke::new(Color::ORANGE_RED, 5.0)
    ));

    view_data.bottom_z -= 0.001;
        
}

pub fn update_edges(
    mut commands: Commands,
    mut edges: Query<(Entity, &GraphEdge, &mut Path)>,
    nodes: Query<&Transform, With<GraphNode>>,
){
    for (edge, data, mut path) in edges.iter_mut() {
        let start = match nodes.get(data.from) {
            Ok(node) => node,
            Err(_) => {
                commands.entity(edge).despawn_recursive();
                continue
            },
        };
        let end = match nodes.get(data.to){
            Ok(node) => node,
            Err(_) => {
                commands.entity(edge).despawn_recursive();
                continue
            },
        };

        *path = GeometryBuilder::build_as(
            &shapes::Line(
                Vec2::new(start.translation.x, start.translation.y),
                Vec2::new(end.translation.x, end.translation.y),
            )
        );
    }
}

// Crude drawing of edges. Deprecated.
pub fn _draw_edges(
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