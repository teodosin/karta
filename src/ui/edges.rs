// Drawing the edges

use bevy::{prelude::*, sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle}, render::render_resource::{ShaderRef, AsBindGroup}};
use bevy_prototype_lyon::{shapes, prelude::{ShapeBundle, GeometryBuilder, Path, Stroke}};

use crate::{graph::{edges::GraphEdge, nodes::GraphDataNode, graph_cam::ViewData, context::PathsToEntitiesIndex}, settings::theme::EDGE_PARENT_COLOR, events::edges::EdgeSpawnedEvent};

use super::nodes::GraphViewNode;

pub struct EdgeUiPlugin;

impl Plugin for EdgeUiPlugin {
    fn build(&self, app: &mut App) {
        let material_plugin = Material2dPlugin::<EdgeMaterial>::default();
        app
            .add_plugins(material_plugin)
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
    mut edge_materials: ResMut<Assets<EdgeMaterial>>,
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
            Stroke::new(edgecol, 4.0)
        ));

        // commands.entity(ev.entity).insert((
        //     MaterialMesh2dBundle {
        //         mesh: default(),
        //         material: edge_materials.add(EdgeMaterial::default().into()),
        //         // material: ColorMaterial::from(edgecol),
        //         transform: Transform {
        //             translation: Vec3::new(0.0, 0.0, view_data.bottom_z),
        //             ..default()
        //         },
        //         ..default()
        //     },
        //     //Fill::color(edgecol),
        //     Path::from(GeometryBuilder::build_as(&line)),
        //     Stroke::new(edgecol, 8.0)
        // ));

        view_data.bottom_z -= 0.001;
    }       
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct EdgeMaterial {
    #[uniform(4)]
    pub color: Color,
}

impl Default for EdgeMaterial {
    fn default() -> Self {
        EdgeMaterial {
            color: Color::rgba(1.0, 1.0, 1.0, 1.0),
        }
    }
}

impl Material2d for EdgeMaterial {
    fn fragment_shader() -> ShaderRef {
        "edge_material.wgsl".into()
    }
}

pub fn update_edges(
    mut edges: Query<(Entity, &GraphEdge, &mut Path)>,
    nodes: Query<&Transform, With<GraphViewNode>>,
    pe_index: Res<PathsToEntitiesIndex>,
){
    for (_edge, data, mut path) in edges.iter_mut() {
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