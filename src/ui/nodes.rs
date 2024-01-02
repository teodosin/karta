use std::{time::Duration, path::PathBuf};

use bevy::{prelude::*, text::Text2dBounds, sprite::Anchor, render::view::RenderLayers};
use bevy_mod_picking::{prelude::*, backends::raycast::RaycastPickable};
use bevy_prototype_lyon::{shapes, prelude::{GeometryBuilder, ShapeBundle, Stroke, StrokeOptions}};
use bevy_tweening::{Tween, EaseFunction, lens::TransformPositionLens, Animator, TweenCompleted, TweeningPlugin};

use crate::{
    graph::{nodes::{GraphNodeEdges, ContextRoot, Pins, GraphDataNode}, node_types::NodeTypes}, 
    events::nodes::{MoveNodesEvent, NodeClickEvent, NodePressedEvent, NodeHoverEvent, NodeSpawnedEvent, NodeHoverStopEvent}
};

use self::node_ui_types::{add_base_node_ui, add_image_node_ui};

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

            .add_systems(PostUpdate, add_node_ui)
            .add_systems(Last, tween_to_target_position)

            .add_systems(PostUpdate, tween_to_target_position_complete)
            .add_systems(PostUpdate, visualise_pinned_position)
            .add_systems(PostUpdate, visualise_root_node)
            .add_systems(PostUpdate, visualise_selected)

            // .add_systems(PreUpdate, outlines_pulse)
            // .add_systems(PreUpdate, visualise_pinned_position)
            .add_systems(PreUpdate, toggle_node_debug_labels)
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
#[derive(Component)]
pub struct GraphViewNode;

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
    new_nodes: Query<(Entity, &GraphDataNode, Option<&TargetPosition>), Added<GraphDataNode>>,
    spawn: Res<GraphStartingPositions>,

    mut commands: Commands,

    mut server: ResMut<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut view_data: ResMut<ViewData>,
    // systems: Res<UiNodeSystemsIndex>,
){
    for (entity, data, tpos) in new_nodes.iter(){

        println!("Node type: {:?}", data.ntype);

        let node = commands.entity(entity).insert((
            RenderLayers::layer(31),
            GraphViewNode,
            Velocity2D::default(),
            PickableBundle {
                pickable: Pickable {
                    should_block_lower: true,
                    should_emit_events: true,
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

        match data.ntype {
            NodeTypes::FileImage => {
                add_image_node_ui(
                    entity, &data, spawn.get_pos(), tpos,
                    &mut commands, &mut server, &mut view_data
                )
            },
            _ => add_base_node_ui(
                entity, &data, spawn.get_pos(), tpos,
                &mut commands, &mut meshes, &mut materials, &mut view_data
            ),
        }

    }
}

/// NAME LABEL
/// This is the text label that will be attached to the node.
/// It will be spawned as a child of the node entity.
pub fn add_node_label(
    commands: &mut Commands,
    entity: &Entity, 
    path: &PathBuf, 
    pos: Vec2,
    top_z: &f32,
){
    
    let name_label = commands.spawn((
        RenderLayers::layer(31),
        NodeLabel,
        Text2dBundle {
            text: Text {
                sections: vec![TextSection::new(
                    &*path.file_name().unwrap().to_string_lossy(),
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
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 100.0 + top_z)),
                ..default()
            },
            ..default()
        },
        Stroke::new(
            crate::settings::theme::OUTLINE_BASE_COLOR, 10.0
            // Color::rgba(0.0, 0.0, 0.0, 0.0), 10.0
        ),
        NodeOutline,
        RaycastPickable,

        On::<Pointer<Over>>::target_component_mut::<Stroke>(move |_over, stroke| {
            stroke.color = crate::settings::theme::OUTLINE_HOVER_COLOR;
            // stroke.color = Color::rgba(0.0, 0.0, 0.0, 0.0);
            stroke.options = StrokeOptions::default().with_line_width(outline_width_hovered);
        }),
        
        On::<Pointer<Out>>::target_component_mut::<Stroke>(move |_out, stroke| {
            stroke.color = crate::settings::theme::OUTLINE_BASE_COLOR;
            // stroke.color = Color::rgba(0.0, 0.0, 0.0, 0.0);
            stroke.options = StrokeOptions::default().with_line_width(outline_width);
        }),
    )).id();
    
    commands.entity(*parent).push_children(&[node_outline]);
}

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


// Debug visualisers
// ----------------------------------------------------------------
pub fn visualise_pinned_position (
    mut gizmos: Gizmos,
    pinned: Query<(&Transform, &Pins)>,
) {
    for (pos, pins) in pinned.iter() {
        if !pins.position {continue}
        gizmos.circle_2d(
            pos.translation.truncate(),
            5.0,
            Color::rgb(0.1, 0.1, 0.9),
        );
    }
}

pub fn visualise_root_node (
    mut gizmos: Gizmos,
    root: Query<&Transform, With<ContextRoot>>,
) {
    for pos in root.iter() {
        gizmos.circle_2d(
            pos.translation.truncate(),
            10.0,
            Color::rgb(0.1, 0.9, 0.1),
        );
        gizmos.circle_2d(
            pos.translation.truncate(),
            13.0,
            Color::rgb(0.1, 0.9, 0.1),
        );
    }
}

pub fn visualise_selected (
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

#[derive(Component)]
pub struct NodeDebugLabel;

pub fn toggle_node_debug_labels(
    mut commands: Commands,

    key_input: ResMut<Input<KeyCode>>,

    mut nodes: Query<(Entity, &GraphNodeEdges, &Transform)>,
    mut labels: Query<Entity, &NodeDebugLabel>,
){
    if !key_input.just_pressed(KeyCode::P){
        return
    }

    if labels.iter_mut().count() > 0 {
        for label in labels.iter_mut(){
            commands.entity(label).despawn_recursive();
        }
        return;
    }
    for (entity, edges, tr) in nodes.iter_mut(){
        // Self entity number
        let entity_number_label = commands.spawn((
            RenderLayers::layer(31),
            NodeDebugLabel,
            Text2dBundle {
                text_anchor: Anchor::TopLeft,
                text: Text {
                    sections: vec![TextSection::new(
                        format!("entity: {:?}", entity),
                        TextStyle {
                            font_size: 32.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    )],
                    alignment: TextAlignment::Left,
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(40.0, -30.0, 10000.0),
                    scale: Vec3::new(0.2, 0.2, 1.0),
                    ..default()
                },
                ..default()
            }
        )).id();
        commands.entity(entity).push_children(&[entity_number_label]);

        // Self transform
        let entity_transform_label = commands.spawn((
            RenderLayers::layer(31),
            NodeDebugLabel,
            Text2dBundle {
                text_anchor: Anchor::TopLeft,
                text: Text {
                    sections: vec![TextSection::new(
                        format!("transform: {:?}", tr.translation),
                        TextStyle {
                            font_size: 32.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    )],
                    alignment: TextAlignment::Left,
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(40.0, -35.0, 10000.0),
                    scale: Vec3::new(0.2, 0.2, 1.0),
                    ..default()
                },
                ..default()
            }
        )).id();
        commands.entity(entity).push_children(&[entity_transform_label]);

        // Number of edges
        let edge_count: &str = &edges.edges.len().to_string();
        let edge_count_label = commands.spawn((
            RenderLayers::layer(31),
            NodeDebugLabel,
            Text2dBundle {
                text_anchor: Anchor::TopLeft,
                text: Text {
                    sections: vec![TextSection::new(
                        format!("edges: {}", edge_count),
                        TextStyle {
                            font_size: 32.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    )],
                    alignment: TextAlignment::Left,
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(40.0, -40.0, 10000.0),
                    scale: Vec3::new(0.2, 0.2, 1.0),
                    ..default()
                },
                ..default()
            }
        )).id();
        commands.entity(entity).push_children(&[edge_count_label]);

        // Edge list
        let mut edge_list_string = String::new();
        for edge in edges.edges.iter(){
            edge_list_string.push_str(&format!("n:{:?} e:{:?}\n", edge.0, edge.1));
        }
        let edge_list_label = commands.spawn((
            RenderLayers::layer(31),
            NodeDebugLabel,
            Text2dBundle {
                text_anchor: Anchor::TopLeft,
                text_2d_bounds: Text2dBounds {
                    size: Vec2::new(140.0, 300.0),
                },
                text: Text {
                    sections: vec![TextSection::new(
                        format!("{}", edge_list_string),
                        TextStyle {
                            font_size: 32.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    )],
                    alignment: TextAlignment::Left,
                    ..default()
                }.with_no_wrap(),
                transform: Transform {
                    translation: Vec3::new(40.0, -45.0, 10000.0),
                    scale: Vec3::new(0.2, 0.2, 1.0),
                    ..default()
                },
                ..default()
            }
        )).id();
        commands.entity(entity).push_children(&[edge_list_label]);
    }
}

// pub fn outlines_pulse(
//     time: Res<Time>,
//     mut outlines: Query<&mut Stroke, With<NodeOutline>>,
// ){
//     let line_width = 5.0 + (time.elapsed_seconds() * 4.0).sin() * 5.0;
//     for mut outline in outlines.iter_mut(){
//         outline.options = StrokeOptions::default().with_line_width(line_width);
//     }
// }


                    