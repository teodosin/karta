// Drawing the edges

use bevy::{prelude::*, sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle}, render::render_resource::{ShaderRef, AsBindGroup}, window::PrimaryWindow};
use bevy_mod_picking::{events::{Pointer, Over, Out}, prelude::On, pointer::{PointerId, PointerLocation}, backend::{PointerHits, HitData}, picking_core::{PickSet, Pickable}};
use bevy_prototype_lyon::{shapes, prelude::{ShapeBundle, GeometryBuilder, Path, Stroke}};
use lyon::lyon_tessellation::StrokeOptions;

use crate::{graph::{edges::GraphEdge, nodes::GraphDataNode, graph_cam::ViewData, context::PathsToEntitiesIndex}, settings::theme::{EDGE_PARENT_COLOR, EDGE_PARENT_HOVER_COLOR}, events::edges::EdgeSpawnedEvent};

use super::nodes::GraphViewNode;

pub struct EdgeUiPlugin;

impl Plugin for EdgeUiPlugin {
    fn build(&self, app: &mut App) {
        let material_plugin = Material2dPlugin::<EdgeMaterial>::default();
        app
            .add_plugins(material_plugin)
            .add_plugins(EdgePickingPlugin)

            .add_systems(PostUpdate, add_edge_ui.after(super::nodes::add_node_ui))
            .add_systems(PostUpdate, update_edges)

            .add_systems(PostUpdate, visualise_edge_transforms)
        ;
    }
}

/// Component containing data only relevant for drawn edges
#[derive(Component, Debug, Default)]
pub struct GraphViewEdge {
    /// Start position in global coordinates
    pub start: Vec2,
    /// End position in global coordinates
    pub end: Vec2,
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
        let hovercol = EDGE_PARENT_HOVER_COLOR;

        commands.entity(ev.entity).insert((
            GraphViewEdge::default(),
            ShapeBundle {
                path: GeometryBuilder::build_as(&line),
                spatial: SpatialBundle {
                    transform: Transform {
                        translation: Vec3::new(0.0, 0.0, view_data.get_z_for_edge()),
                        ..default()
                    },
                    ..default()
                },
                ..default()
            },
            Stroke::new(edgecol, 7.0),
            Pickable {
                should_block_lower: true,
                should_emit_events: true, 
            },

            On::<Pointer<Over>>::target_component_mut::<Stroke>(move |_over, stroke| {
                stroke.color = hovercol;
                // stroke.color = Color::rgba(0.0, 0.0, 0.0, 0.0);
                stroke.options = StrokeOptions::default().with_line_width(8.);
            }),
            
            On::<Pointer<Out>>::target_component_mut::<Stroke>(move |_out, stroke| {
                stroke.color = edgecol;
                // stroke.color = Color::rgba(0.0, 0.0, 0.0, 0.0);
                stroke.options = StrokeOptions::default().with_line_width(7.);
            }),
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
    mut edges: Query<(Entity, &GraphEdge, &mut Path, &mut GraphViewEdge), Without<GraphViewNode>>,
    nodes: Query<&Transform, With<GraphViewNode>>,
    pe_index: Res<PathsToEntitiesIndex>,
){
    for (_edge, data, mut path, mut ends) in edges.iter_mut() {
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

        ends.start = Vec2::new(start.translation.x, start.translation.y);
        ends.end = Vec2::new(end.translation.x, end.translation.y);

    }
}

fn visualise_edge_transforms(
    edges: Query<&GlobalTransform, With<GraphViewEdge>>,
    mut gizmos: Gizmos,
){
    for gtform in edges.iter(){
        gizmos.circle_2d(
            gtform.translation().xy(),
            2.0,
            Color::rgba(1.0, 1.0, 1.0, 1.0),
        );
    }
}

/// Edge picking plugin
/// Custom picking plugin for edges. Currently
pub struct EdgePickingPlugin;

impl Plugin for EdgePickingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(PreUpdate, edge_picking.in_set(PickSet::Backend))
            .add_systems(PostUpdate, picking_debug)
        ;
    }
}

fn picking_debug(
    map: ResMut<bevy_mod_picking::focus::HoverMap>,
    key: Res<Input<KeyCode>>,
){
    if !key.just_pressed(KeyCode::H) {
        return
    }
    let printable = map.iter().next().unwrap().1;
    for (_ , hover) in printable.iter() {
        println!("Hover: {:?}", hover.depth);
    }
}

pub fn edge_picking(
    pointers: Query<(&PointerId, &PointerLocation)>,
    cameras: Query<(Entity, &Camera, &GlobalTransform)>,
    primary_window: Query<Entity, With<PrimaryWindow>>,

    mut edges: Query<(
        Entity,
        &Transform,
        &GraphViewEdge,
        Option<&Pickable>,
        &ViewVisibility,
    )>,

    mut output: EventWriter<PointerHits>,
){
    let threshold = 25.0;
    
    for (pointer, location) in pointers.iter().filter_map(|(pointer, pointer_location)| {
        pointer_location.location().map(|loc| (pointer, loc))
    }) {
        let mut blocked = false;
        let Some((cam_entity, camera, cam_transform)) = cameras
            .iter()
            .filter(|(_, camera, _)| camera.is_active)
            .find(|(_, camera, _)| {
                camera
                    .target
                    .normalize(Some(primary_window.single()))
                    .unwrap()
                    == location.target
            })
        else {
            continue;
        };

        let Some(cursor_pos_world) = camera.viewport_to_world_2d(cam_transform, location.position)
        else {
            continue;
        };

        let mut picks_presort: Vec<(Entity, f32, f32)> = edges
            .iter()
            .filter(|(.., visibility)| visibility.get())
            .filter_map(|(entity, edgetr, edge, pickable, ..)| {
                // Calculate the distance from the pointer to the edge
                if blocked {
                    return None;
                }
                
                let distance = distance_to_edge(&cursor_pos_world, edge);
                let within_bounds = distance < threshold;
                blocked = within_bounds && pickable.map(|p| p.should_block_lower) != Some(false);
                
                within_bounds.then_some((
                    entity,
                    distance,
                    edgetr.translation.z,
                ))

            })
            .collect();

        // Sort the picks by distance
        picks_presort.sort_by(|(_, adist, _), (_, bdist, _)| adist.partial_cmp(&bdist).unwrap());
        let picks_sort: Vec<(Entity, HitData)> = picks_presort.iter().map(|(entity, dist, z)| {
            // println!("Edge z: {:?}", z);
            (*entity, HitData::new(cam_entity, -dist, None, None))
        })
        .collect();

        let order = camera.order as f32;
        output.send(PointerHits::new(*pointer, picks_sort, order))
    }
}

fn distance_to_edge(cursor_pos_world: &Vec2, edge: &GraphViewEdge) -> f32 {
    // Get the start and end points of the edge
    let p1 = edge.start;
    let p2 = edge.end;

    // Calculate the square of the distance from the start to the end point
    let line_sq = p1.distance_squared(p2);

    if line_sq == 0.0 {
        // The edge is a point, return the distance from the cursor to this point
        return cursor_pos_world.distance(p1);
    }

    // Consider the line extending the edge, parameterized as p1 + t (p2 - p1).
    // We find the projection of the cursor point onto this line.
    // It falls where t = [(cursor_pos_world-p1) . (p2-p1)] / |p2-p1|^2
    let t = ((*cursor_pos_world - p1).dot(p2 - p1)) / line_sq;

    if t < 0.0 {
        // The projection falls on the segment p1 p1
        cursor_pos_world.distance(p1)
    } else if t > 1.0 {
        // The projection falls on the segment p2 p2
        cursor_pos_world.distance(p2)
    } else {
        // The projection falls on the segment p1 p2
        let projection = p1 + t * (p2 - p1);
        cursor_pos_world.distance(projection)
    }
}