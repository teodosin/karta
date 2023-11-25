use bevy::{prelude::*, sprite::MaterialMesh2dBundle, text::Text2dBounds};
use bevy_mod_picking::{prelude::*, backends::raycast::RaycastPickable};
use bevy_prototype_lyon::{shapes, prelude::{GeometryBuilder, ShapeBundle, Stroke, StrokeOptions}};
use rand::Rng;

use crate::{
    graph::{nodes::{GraphDataNode, PinnedToPosition}, graph_cam::ViewData, context::Selected, node_types::NodeTypes}, 
    events::nodes::{MoveNodesEvent, NodeClickEvent, NodePressedEvent, NodeHoverEvent, NodeSpawnedEvent}
};

use self::node_ui_types::{add_base_node_ui, add_image_node_ui};

mod node_ui_types;

pub struct NodesUiPlugin;

impl Plugin for NodesUiPlugin {
    fn build(&self, app: &mut App) {
        app
            // .insert_resource(UiNodeSystemsIndex::default())
            // .add_systems(PreStartup, setup_node_ui_systems)
            .add_systems(PreUpdate, handle_outline_hover)

            .add_systems(PostUpdate, add_node_ui)
            .add_systems(PostUpdate, visualise_pinned_position)

            // .add_systems(PreUpdate, outlines_pulse)
            // .add_systems(PreUpdate, visualise_pinned_position)
        ;
    }
}

// Component Definitions
// ----------------------------------------------------------------

// Basic marker component to identify all nodes that have a graphical representation
// in the graph. 
#[derive(Component)]
pub struct GraphViewNode;

// Marker component for all nodes that have the interactive outline. 
#[derive(Component)]
pub struct NodeOutline;

// Marker component for all nodes that have a visible label. Nodes with this
// component will have a text label attached to them.
#[derive(Component)]
pub struct NodeLabel;

// Component to store the current velocity of a node. 
// This component might get abstracted out at some point, because it is very generic. 
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

// Component to store the scale of a node. This should also affect the simulation,
// but I don't know how that should behave just yet. So for now there will be a 
// separate "radius" variable that will be used for the simulation.
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
    mut events: EventReader<NodeSpawnedEvent>,

    mut commands: Commands,

    mut server: ResMut<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut view_data: ResMut<ViewData>,
    // systems: Res<UiNodeSystemsIndex>,
){
    
    for ev in events.read(){

        println!("Node type: {:?}", ev.ntype);

        commands.entity(ev.entity).insert((
            GraphViewNode,
            Velocity2D::default(),
            PickableBundle::default(),
            
            On::<Pointer<Drag>>::send_event::<MoveNodesEvent>(),
            On::<Pointer<Click>>::send_event::<NodeClickEvent>(),
            On::<Pointer<Down>>::send_event::<NodePressedEvent>(),
            On::<Pointer<Over>>::send_event::<NodeHoverEvent>(),
            
            On::<Pointer<DragStart>>::target_insert(Selected),
            On::<Pointer<DragEnd>>::target_remove::<Selected>(),
            On::<Pointer<Deselect>>::target_remove::<Selected>(),
        ));

        match ev.ntype {
            NodeTypes::FileImage => {
                add_image_node_ui(&ev, &mut commands, &mut server, &mut view_data)
            },
            _ => add_base_node_ui(&ev, &mut commands, &mut meshes, &mut materials, &mut view_data),
        }

    }
}

// NAME LABEL
// This is the text label that will be attached to the node.
// It will be spawned as a child of the node entity.
pub fn add_node_label(
    commands: &mut Commands,
    ev: &NodeSpawnedEvent, 
    pos: Vec2,
    top_z: &f32,
){
    
    let name_label = commands.spawn((
        NodeLabel,
        Text2dBundle {
            text: Text {
                sections: vec![TextSection::new(
                    &*ev.name.to_string_lossy(),
                    TextStyle {
                        font_size: 20.0,
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
                translation: Vec3::new(pos.x, pos.y, 10000.0 + top_z),
                ..default()
            },
            text_anchor: bevy::sprite::Anchor::CenterLeft,
            ..default()
        }
    )).id();
    
    commands.entity(ev.entity).push_children(&[name_label]);
}

// OUTLINE
// ----------------------------------------------------------------
// This is the hoverable, interactable outline from which edges are created.
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
        ShapeBundle {
            path: outline_path,
            spatial: SpatialBundle {
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1000.0 + top_z)),
                ..default()
            },
            ..default()
        },
        Stroke::new(
            crate::settings::theme::OUTLINE_BASE_COLOR, 10.0
        ),
        NodeOutline,
        
        //RaycastPickable::default(),

        On::<Pointer<Over>>::target_component_mut::<Stroke>(move |_over, stroke| {
            stroke.color = crate::settings::theme::OUTLINE_HOVER_COLOR;
            stroke.options = StrokeOptions::default().with_line_width(outline_width_hovered);
        }),
        
        On::<Pointer<Out>>::target_component_mut::<Stroke>(move |_out, stroke| {
            stroke.color = crate::settings::theme::OUTLINE_BASE_COLOR;
            stroke.options = StrokeOptions::default().with_line_width(outline_width);
        }),
    )).id();
    
    commands.entity(*parent).push_children(&[node_outline]);
}

pub fn handle_outline_hover(
    // ev_spawn: EventReader<Node>,
    _nodes: Query<&Transform, With<GraphDataNode>>,
    
    // mut commands: Commands,
){
    
}

// Debug visualisers
// ----------------------------------------------------------------
pub fn visualise_pinned_position (
    mut gizmos: Gizmos,
    pinned: Query<&Transform, With<PinnedToPosition>>,
) {
    for pos in pinned.iter() {
        gizmos.circle_2d(
            pos.translation.truncate(),
            5.0,
            Color::rgb(0.1, 0.1, 0.9),
        );
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


                    