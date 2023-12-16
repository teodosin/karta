// Drawing the edges

use bevy::prelude::*;
use bevy_prototype_lyon::{shapes, prelude::{ShapeBundle, GeometryBuilder, Path, Stroke}};

use crate::{graph::{edges::GraphEdge, nodes::GraphDataNode, graph_cam::ViewData, context::PathsToEntitiesIndex}, settings::theme::EDGE_PARENT_COLOR, events::edges::EdgeSpawnedEvent};

use super::nodes::GraphViewNode;

pub struct EdgeUiPlugin;

impl Plugin for EdgeUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(PostUpdate, add_edge_ui.after(super::nodes::add_node_ui))
            .add_systems(PostUpdate, update_edges)
            //.add_systems(PostUpdate, _draw_edges)
        ;
    }
}

pub fn add_edge_ui(
    mut events: EventReader<EdgeSpawnedEvent>,
    mut commands: Commands,
    mut view_data: ResMut<ViewData>,
){
    for ev in events.read() {
        let line = shapes::Line(
            Vec2::ZERO, Vec2::ZERO
        );

        let edgecol = EDGE_PARENT_COLOR;

        commands.entity(ev.entity).insert((
            ShapeBundle {
                path: GeometryBuilder::build_as(&line),
                spatial: SpatialBundle {
                    transform: Transform {
                        translation: Vec3::new(0.0, 0.0, view_data.bottom_z),
                        ..default()
                    },
                    ..default()
                },
                ..default()
            },
            //Fill::color(edgecol),
            Stroke::new(edgecol, 8.0)
        ));

        view_data.bottom_z -= 0.001;
    }
        
}

pub fn update_edges(
    mut commands: Commands,
    mut edges: Query<(Entity, &GraphEdge, &mut Path)>,
    nodes: Query<&Transform, With<GraphViewNode>>,
    pe_index: Res<PathsToEntitiesIndex>,
){
    for (edge, data, mut path) in edges.iter_mut() {
        let source_entity = match pe_index.0.get(&data.source){
            Some(entity) => entity,
            None => {
                continue
            },
        };
        let target_entity = match pe_index.0.get(&data.target){
            Some(entity) => entity,
            None => {
                continue
            },
        };
        let start = match nodes.get(*source_entity) {
            Ok(node) => node,
            Err(_) => {
                // commands.entity(edge).despawn_recursive();
                continue
            },
        };
        let end = match nodes.get(*target_entity){
            Ok(node) => node,
            Err(_) => {
                // commands.entity(edge).despawn_recursive();
                continue
            },
        };
        // Check that all positions are valid
        if !start.translation.x.is_finite() || !start.translation.y.is_finite() || !end.translation.x.is_finite() || !end.translation.y.is_finite() {
            // commands.entity(edge).despawn_recursive();
            continue
        }

        *path = GeometryBuilder::build_as(
            &shapes::Line(
                Vec2::new(start.translation.x, start.translation.y),
                Vec2::new(end.translation.x, end.translation.y),
            )
        );
    }
}