use std::{time::Duration, path::PathBuf};

use bevy::{prelude::*, text::Text2dBounds, sprite::Anchor, render::view::RenderLayers, window::PrimaryWindow};
use bevy_mod_picking::{prelude::*, backends::raycast::RaycastPickable, backend::{PointerHits, HitData}};
use bevy_prototype_lyon::{shapes, prelude::{GeometryBuilder, ShapeBundle, Stroke, StrokeOptions}};
use bevy_tweening::{Tween, EaseFunction, lens::TransformPositionLens, Animator, TweenCompleted, TweeningPlugin};

use crate::{
    events::node_events::*, settings::theme::*, prelude::{GraphEntity, Pins, pointer::InputData}, 
};

use self::node_ui_types::add_base_node_ui;

use super::graph_cam::ViewData;

mod node_ui_types;

pub struct NodesUiPlugin;

impl Plugin for NodesUiPlugin {
    fn build(&self, app: &mut App) {
        app
            // .insert_resource(UiNodeSystemsIndex::default())
            // .add_systems(PreStartup, setup_node_ui_systems)
            .add_plugins(TweeningPlugin)

            .insert_resource(GraphStartingPositions::default())

            .add_systems(PreUpdate, node_picking.in_set(picking_core::PickSet::Backend))

            .add_systems(Update, move_node_selection)
            .add_systems(Update, toggle_outline_on_node_select)

            .add_systems(PostUpdate, add_node_ui)
            .add_systems(Last, tween_to_target_position)

            .add_systems(PostUpdate, tween_to_target_position_complete)
            
            // .add_systems(PostUpdate, visualise_pinned_position)
            // .add_systems(PostUpdate, visualise_root_node)
            // .add_systems(PostUpdate, visualise_selected)
            // .add_systems(PostUpdate, visualise_visitors)

            // .add_systems(PreUpdate, outlines_pulse)
            // .add_systems(PreUpdate, visualise_pinned_position)
            // .add_systems(PreUpdate, toggle_node_debug_labels)
        ;
    }
}

/// Resource to store the default spawn position(s) for nodes in the graph. 
/// Modify this resource if you want to spawn nodes somewhere other than the world origin. 
/// This resource will likely be extended in the future to allow for more complex/specific
/// spawn layouts such as grids. 
#[derive(Resource, Default)]
pub struct GraphStartingPositions {
    position: Vec2,
}

impl GraphStartingPositions {
    pub fn get_pos(&self) -> Vec2 {
        self.position
    }

    pub fn set_pos(&mut self, pos: Vec2) {
        println!("Setting spawn position to {:?}", pos);
        self.position = pos;
    }
}

// Component Definitions
// ----------------------------------------------------------------

/// Basic marker component to identify all nodes that have a graphical representation
/// in the graph. 
/// Also stores the dimensions of the view node for the purpose of the custom picking backend.
#[derive(Component)]
pub struct GraphViewNode {
    target: Entity,
}

impl GraphViewNode {
    pub fn get_target(&self) -> Entity {
        self.target
    }
}

#[derive(Component)]
pub enum ViewNodeShape {
    Circle(f32),
    Rectangle(Vec2),
}

/// Marker component for the node outline. 
#[derive(Component)]
pub struct NodeOutline;

/// Marker component for node name labels
#[derive(Component)]
pub struct NodeLabel;

/// Component storing the target position of a node. Nodes with this component will be 
/// ignored by the simulation. When the node reaches the target position, this component 
/// will be removed.
#[derive(Component)]
pub struct TargetPosition {
    pub position: Vec2,
}

/// Component to store the current velocity of a node. 
/// This component might get abstracted out at some point, because it is very generic. 
#[derive(Component)]
pub struct Velocity2D {
    pub velocity: Vec2,
}

impl Default for Velocity2D {
    fn default() -> Self {
        Velocity2D {
            velocity: Vec2::new(0.0, 0.0),
        }
    }
}

/// Component to store the scale of a node. This should also affect the simulation,
/// but the precise behavior hasn't been decided. So for now there will be a 
/// separate "radius" variable that will be used for the simulation.
#[derive(Component)]
pub struct ViewNodeScale {
    pub scale: Vec2,
    pub radius: f32,
}

impl Default for ViewNodeScale {
    fn default() -> Self {
        ViewNodeScale {
            scale: Vec2::from((1.0, 1.0)),
            radius: 25.0,
        }
    }
}

// Component to store the radius of a node. This is used for the simulation.


// TODO: Convert to One-Shot System
// Is that even possible? This function requires input parameters. 
pub fn add_node_ui(
    new_nodes: Query<(Entity, &GraphEntity, Option<&Name>, Option<&TargetPosition>), Added<GraphEntity>>,
    spawn: Res<GraphStartingPositions>,

    mut commands: Commands,

    mut server: ResMut<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut view_data: ResMut<ViewData>,
    // systems: Res<UiNodeSystemsIndex>,
){
    for (entity, data, name, tpos) in new_nodes.iter(){

        // println!("Node type: {:?}", data.ntype);

        let node = commands.spawn((
            SpatialBundle {
                transform: Transform {
                    translation: Vec3::new(spawn.get_pos().x, spawn.get_pos().y, 0.0),
                    scale: Vec3::new(1.0, 1.0, 1.0),
                    ..default()
                },
                ..default()
            },
            RenderLayers::layer(31),
            GraphViewNode {
                target: entity,
            },
            Pins::default(),
            Velocity2D::default(),

            PickableBundle {
                pickable: Pickable {
                    is_hoverable: true,
                    should_block_lower: true,
                },
                ..default()
            },
            RaycastPickable,
            
            On::<Pointer<Drag>>::send_event::<MoveNodesEvent>(),
            On::<Pointer<Click>>::send_event::<NodeClickEvent>(),
            On::<Pointer<Down>>::send_event::<NodePressedEvent>(),
            On::<Pointer<Over>>::send_event::<NodeHoverEvent>(),
            On::<Pointer<Out>>::send_event::<NodeHoverStopEvent>(),
            
        )).id();

        commands.entity(node).insert(
            On::<Pointer<DragStart>>::target_component_mut::<PickSelection>(| _drag, pick | {
                pick.is_selected = true;
            }),
        );

        // match data.ntype {
        //     NodeTypes::FileImage => {
        //         add_image_node_ui(
        //             entity, &data, spawn.get_pos(), tpos,
        //             &mut commands, &mut server, &mut view_data
        //         )
        //     },
        //     _ => add_base_node_ui(
        //         entity, &data, spawn.get_pos(), tpos,
        //         &mut commands, &mut meshes, &mut materials, &mut view_data
        //     ),
        // }

        add_base_node_ui(
            node, &data, name, spawn.get_pos(), tpos,
            &mut commands, &mut meshes, &mut materials, &mut view_data
        )

    }
}

/// NAME LABEL
/// This is the text label that will be attached to the node.
/// It will be spawned as a child of the node entity.
pub fn add_node_label(
    commands: &mut Commands,
    entity: &Entity, 
    name: &Name, 
    pos: Vec2,
    top_z: &f32,
){
    
    let name_label = commands.spawn((
        RenderLayers::layer(31),
        NodeLabel,
        Text2dBundle {
            text: Text {
                sections: vec![TextSection::new(
                    name.as_str(),
                    TextStyle {
                        font_size: 100.0,
                        color: Color::WHITE,
                        ..default()
                    },
                )],
                ..default()
            },
            text_2d_bounds: Text2dBounds {
                size: Vec2::new(400.0, 200.0),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(pos.x, pos.y, 1000.0 + top_z),
                scale: Vec3::new(0.2, 0.2, 1.0),
                ..default()
            },
            text_anchor: bevy::sprite::Anchor::CenterLeft,
            ..default()
        }
    )).id();
    
    commands.entity(*entity).push_children(&[name_label]);
}

/// OUTLINE
/// Interactive outline for nodes. Spawned as a child of the node entity. 
pub fn add_node_base_outline(
    commands: &mut Commands,
    parent: &Entity,
    radius: f32,
    top_z: &f32,
){
    
    let outline_width = 10.0;
    let outline_width_hovered = 12.0;
    
    let outline_shape = shapes::Circle {
        radius: radius + outline_width / 2.,
        center: Vec2::from((0.0, 0.0)),
    };

    let outline_path = GeometryBuilder::build_as(&outline_shape);
    
    let node_outline = commands.spawn((
        RenderLayers::layer(31),
        ShapeBundle {
            path: outline_path,
            spatial: SpatialBundle {
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, top_z + 0.001)),
                ..default()
            },
            ..default()
        },
        Stroke::new(
            OUTLINE_BASE_COLOR, 10.0
            // Color::rgba(0.0, 0.0, 0.0, 0.0), 10.0
        ),
        NodeOutline,
        PickSelection::default(),
        NoDeselect,
        RaycastPickable,

        On::<Pointer<Over>>::target_component_mut::<Stroke>(move |_over, stroke| {
            stroke.color = OUTLINE_HOVER_COLOR;
            // stroke.color = Color::rgba(0.0, 0.0, 0.0, 0.0);
            stroke.options = StrokeOptions::default().with_line_width(outline_width_hovered);
        }),
        
        On::<Pointer<Out>>::target_component_mut::<Stroke>(move |_out, stroke| {
            stroke.color = OUTLINE_BASE_COLOR;
            // stroke.color = Color::rgba(0.0, 0.0, 0.0, 0.0);
            stroke.options = StrokeOptions::default().with_line_width(outline_width);
        }),
    )).id();
    
    commands.entity(*parent).push_children(&[node_outline]);
}

// Positional tweening systems
// --------------------------------------------------------------------------------------------------------------------------------------------
pub fn tween_to_target_position(
    mut commands: Commands,
    mut nodes: Query<(Entity, &mut Transform, &TargetPosition), (With<GraphViewNode>, Added<TargetPosition>)>,
){
    if nodes.iter_mut().count() == 0 {
        return
    }
    println!("Tween to target position triggered, length: {}", nodes.iter_mut().count());
    for (entity, transform, target) in nodes.iter_mut() {
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs_f32(0.35),
            TransformPositionLens {
                start: transform.translation,
                end: Vec3::new(target.position.x, target.position.y, transform.translation.z),
            }
        )
        .with_completed_event(1);

        println!("Should be tweening from {:?} to {:?}", transform.translation, target.position);

        commands.entity(entity).insert(Animator::new(tween));

    }
}

pub fn tween_to_target_position_complete(
    mut commands: Commands,
    mut event: EventReader<TweenCompleted>,
){
    for ev in event.read(){
        if true {
            println!("Tween complete");
            commands.entity(ev.entity).remove::<TargetPosition>();
        }
    }
}

// Node interaction systems
// --------------------------------------------------------------------------------------------------------------------------------------------
pub fn move_node_selection(
    mut ev_mouse_drag: EventReader<MoveNodesEvent>,
    mouse: Res<ButtonInput<MouseButton>>,
    input_data: Res<InputData>,
    mut query: Query<(Entity, &GraphViewNode, &mut Transform, &PickSelection), /*With<Selected>*/>,
) {

    if mouse.just_pressed(MouseButton::Left) {
        for (_entity, _node, mut transform, selection) in query.iter_mut() {
            if !selection.is_selected {continue};
            // transform.translation.z = 60.0;
        }
    }

    for _ev in ev_mouse_drag.read() {
        if mouse.pressed(MouseButton::Left){
            for (_entity, _node, mut transform, selection) in query.iter_mut() {
                if !selection.is_selected {continue};
                transform.translation.x += input_data.cursor_world_current_position().x - input_data.cursor_world_previous_position().x;
                transform.translation.y += input_data.cursor_world_current_position().y - input_data.cursor_world_previous_position().y;     
            }
        }
        break
    }
}

pub fn toggle_outline_on_node_select(
    nodes: Query<(&PickSelection, &Children), Changed<PickSelection>>,
    mut outlines: Query<&mut Visibility, With<NodeOutline>>,
){
    for (selection, children) in nodes.iter(){
        for child in children.iter(){
            let outline = outlines.get_mut(*child);
            match outline {
                Ok(mut outline) => {
                    if selection.is_selected {
                        *outline = Visibility::Visible;
                    } else {
                        *outline = Visibility::Hidden;
                    }
                },
                Err(_) => continue
            }
        }
    }
}

// Custom picking backend because for some reason sprite picking is absolutely broken.
// --------------------------------------------------------------------------------------------------------------------------------------------
/// Custom picking backend for nodes. Intended to become deprecated once Bevy UI
/// is more developed and the node system gets ported over to it.
fn node_picking(
    pointers: Query<(&PointerId, &PointerLocation)>,
    cameras: Query<(Entity, &Camera, &GlobalTransform, &OrthographicProjection)>,
    primary_window: Query<Entity, With<PrimaryWindow>>,

    nodes: Query<(
        Entity,
        &GlobalTransform,
        &ViewNodeShape,
        Option<&Pickable>,
        &ViewVisibility,
    )>,

    mut output: EventWriter<PointerHits>,
){    
    let mut sorted_nodes: Vec<_> = nodes.iter().collect();
    sorted_nodes.sort_by(|a, b| {
        (b.1.translation().z)
            .partial_cmp(&a.1.translation().z)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for (pointer, location) in pointers.iter().filter_map(|(pointer, pointer_location)| {
        pointer_location.location().map(|loc| (pointer, loc))
    }) {
        let mut blocked = false;
        let Some((cam_entity, camera, cam_transform, cam_ortho)) = cameras
            .iter()
            .filter(|(_, camera, _, _)| camera.is_active)
            .find(|(_, camera, _,_)| {
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

        let near_clipping_plane = cam_ortho.near;

        let picks: Vec<(Entity, HitData)> = sorted_nodes
            .iter()
            .copied()
            .filter(|(.., visibility)| visibility.get())
            .filter_map(|(entity, transform, shape, pickable, ..)| {
                if blocked {
                    return None;
                }

                // Hit box in sprite coordinate system
                let extents = match shape {
                    ViewNodeShape::Circle(radius) => Vec2::new(*radius, *radius),
                    ViewNodeShape::Rectangle(extents) => *extents,
                };

                // let center = transform.translation().truncate();
                let center = Vec2::ZERO;
                let rect = Rect::from_center_half_size(center, extents / 2.0);

                // Transform cursor pos to sprite coordinate system
                let cursor_pos_sprite = transform
                    .affine()
                    .inverse()
                    .transform_point3((cursor_pos_world, 0.0).into());

                let is_cursor_in_sprite = rect.contains(cursor_pos_sprite.truncate());
                blocked =
                    is_cursor_in_sprite && pickable.map(|p| p.should_block_lower) != Some(false);

                is_cursor_in_sprite.then_some((
                    entity,
                    HitData::new(cam_entity, -near_clipping_plane -transform.translation().z, None, None),
                ))
            })
            .collect();

        let order = camera.order as f32;
        output.send(PointerHits::new(*pointer, picks, order));
    }
}

// Debug visualisers
// --------------------------------------------------------------------------------------------------------------------------------------------
pub fn visualise_pinned_position (
    mut gizmos: Gizmos,
    pinned: Query<(&Transform, &Pins)>,
) {
    for (pos, pins) in pinned.iter() {
        if !pins.position {continue}
        gizmos.circle_2d(
            pos.translation.truncate() + Vec2::new(0.0, 30.0),
            5.0,
            Color::rgb(0.9, 0.9, 0.9),
        );
    }
}

fn visualise_selected (
    mut gizmos: Gizmos,
    // selected: Query<&Transform, With<Selected>>,
    selected: Query<(&Transform, &PickSelection)>,

){
    for (pos, pick) in selected.iter() {
        if !pick.is_selected {continue};
        gizmos.line_2d(
            pos.translation.truncate() + Vec2::new(-20.0, 0.0),
            pos.translation.truncate() + Vec2::new(20.0, 0.0),
            Color::rgb(0.9, 0.1, 0.1),
        );
    }
}

// fn visualise_visitors ( 
//     mut gizmos: Gizmos,
//     visitors: Query<&Transform, With<Visitor>>,
// ){
//     for node in visitors.iter() {
//         gizmos.circle_2d(
//             node.translation.truncate() + Vec2::new(-25.0, -25.0),
//             5.0,
//             Color::rgb(0.1, 0.9, 0.1),
//         );
//     }
// }

// #[derive(Component)]
// pub struct NodeDebugLabel;

// pub fn toggle_node_debug_labels(
//     mut commands: Commands,

//     key_input: ResMut<Input<KeyCode>>,

//     mut nodes: Query<(Entity, &GraphNodeEdges, &Transform)>,
//     mut labels: Query<Entity, &NodeDebugLabel>,
// ){
//     if !key_input.just_pressed(KeyCode::P){
//         return
//     }

//     if labels.iter_mut().count() > 0 {
//         for label in labels.iter_mut(){
//             commands.entity(label).despawn_recursive();
//         }
//         return;
//     }
//     for (entity, edges, tr) in nodes.iter_mut(){
//         // Self entity number
//         let entity_number_label = commands.spawn((
//             RenderLayers::layer(31),
//             NodeDebugLabel,
//             Text2dBundle {
//                 text_anchor: Anchor::TopLeft,
//                 text: Text {
//                     sections: vec![TextSection::new(
//                         format!("entity: {:?}", entity),
//                         TextStyle {
//                             font_size: 32.0,
//                             color: Color::WHITE,
//                             ..default()
//                         },
//                     )],
//                     alignment: TextAlignment::Left,
//                     ..default()
//                 },
//                 transform: Transform {
//                     translation: Vec3::new(40.0, -30.0, 10000.0),
//                     scale: Vec3::new(0.2, 0.2, 1.0),
//                     ..default()
//                 },
//                 ..default()
//             }
//         )).id();
//         commands.entity(entity).push_children(&[entity_number_label]);

//         // Self transform
//         let entity_transform_label = commands.spawn((
//             RenderLayers::layer(31),
//             NodeDebugLabel,
//             Text2dBundle {
//                 text_anchor: Anchor::TopLeft,
//                 text: Text {
//                     sections: vec![TextSection::new(
//                         format!("transform: {:?}", tr.translation),
//                         TextStyle {
//                             font_size: 32.0,
//                             color: Color::WHITE,
//                             ..default()
//                         },
//                     )],
//                     alignment: TextAlignment::Left,
//                     ..default()
//                 },
//                 transform: Transform {
//                     translation: Vec3::new(40.0, -35.0, 10000.0),
//                     scale: Vec3::new(0.2, 0.2, 1.0),
//                     ..default()
//                 },
//                 ..default()
//             }
//         )).id();
//         commands.entity(entity).push_children(&[entity_transform_label]);

//         // Number of edges
//         let edge_count: &str = &edges.edges.len().to_string();
//         let edge_count_label = commands.spawn((
//             RenderLayers::layer(31),
//             NodeDebugLabel,
//             Text2dBundle {
//                 text_anchor: Anchor::TopLeft,
//                 text: Text {
//                     sections: vec![TextSection::new(
//                         format!("edges: {}", edge_count),
//                         TextStyle {
//                             font_size: 32.0,
//                             color: Color::WHITE,
//                             ..default()
//                         },
//                     )],
//                     alignment: TextAlignment::Left,
//                     ..default()
//                 },
//                 transform: Transform {
//                     translation: Vec3::new(40.0, -40.0, 10000.0),
//                     scale: Vec3::new(0.2, 0.2, 1.0),
//                     ..default()
//                 },
//                 ..default()
//             }
//         )).id();
//         commands.entity(entity).push_children(&[edge_count_label]);

//         // Edge list
//         let mut edge_list_string = String::new();
//         for edge in edges.edges.iter(){
//             edge_list_string.push_str(&format!("n:{:?} e:{:?}\n", edge.0, edge.1));
//         }
//         let edge_list_label = commands.spawn((
//             RenderLayers::layer(31),
//             NodeDebugLabel,
//             Text2dBundle {
//                 text_anchor: Anchor::TopLeft,
//                 text_2d_bounds: Text2dBounds {
//                     size: Vec2::new(140.0, 300.0),
//                 },
//                 text: Text {
//                     sections: vec![TextSection::new(
//                         format!("{}", edge_list_string),
//                         TextStyle {
//                             font_size: 32.0,
//                             color: Color::WHITE,
//                             ..default()
//                         },
//                     )],
//                     alignment: TextAlignment::Left,
//                     ..default()
//                 }.with_no_wrap(),
//                 transform: Transform {
//                     translation: Vec3::new(40.0, -45.0, 10000.0),
//                     scale: Vec3::new(0.2, 0.2, 1.0),
//                     ..default()
//                 },
//                 ..default()
//             }
//         )).id();
//         commands.entity(entity).push_children(&[edge_list_label]);
//     }
// }

// pub fn outlines_pulse(
//     time: Res<Time>,
//     mut outlines: Query<&mut Stroke, With<NodeOutline>>,
// ){
//     let line_width = 5.0 + (time.elapsed_seconds() * 4.0).sin() * 5.0;
//     for mut outline in outlines.iter_mut(){
//         outline.options = StrokeOptions::default().with_line_width(line_width);
//     }
// }


                    