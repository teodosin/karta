use bevy::{prelude::*, sprite::MaterialMesh2dBundle, text::Text2dBounds};
use bevy_mod_picking::{prelude::*, backends::raycast::RaycastPickable};
use bevy_prototype_lyon::{shapes, prelude::{GeometryBuilder, ShapeBundle, Stroke, StrokeOptions}};
use rand::Rng;

use crate::{
    graph::{nodes::{GraphDataNode, PinnedToPosition}, graph_cam::ViewData, context::Selected, node_types::NodeTypes}, 
    events::nodes::{MoveNodesEvent, NodeClickEvent, NodePressedEvent, NodeHoverEvent, NodeSpawnedEvent}
};

pub struct NodesUiPlugin;

impl Plugin for NodesUiPlugin {
    fn build(&self, app: &mut App) {
        app
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

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut view_data: ResMut<ViewData>,
){
    
    for ev in events.read(){
        
        // let full_path = format!("{}/{}", ev.path, ev.name);
        
        // Positions are slightly randomized to avoid nodes being spawned on top of each other
        let mut rng = rand::thread_rng();
        
        let radius = match ev.ntype {
            NodeTypes::FileImage => 70.0,
            NodeTypes::Folder => 50.0,
            _ => 25.0,
        };

        commands.entity(ev.entity).insert((
            GraphViewNode,
            Velocity2D::default(),
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(radius).into()).into(),
                material: materials.add(ColorMaterial::from(Color::rgb(0.3, 0.0, 0.0))),
                transform: Transform::from_translation(Vec3::new(
                    ev.position.x + rng.gen_range(-10.0..10.0),
                    ev.position.y + rng.gen_range(-10.0..10.0),
                    view_data.top_z,
                )),
                ..default()
            },
            PickableBundle::default(),
            RaycastPickable::default(),
            
            On::<Pointer<Drag>>::send_event::<MoveNodesEvent>(),
            On::<Pointer<Click>>::send_event::<NodeClickEvent>(),
            On::<Pointer<Down>>::send_event::<NodePressedEvent>(),
            On::<Pointer<Over>>::send_event::<NodeHoverEvent>(),
            
            On::<Pointer<DragStart>>::target_insert(Selected),
            On::<Pointer<DragEnd>>::target_remove::<Selected>(),
            On::<Pointer<Deselect>>::target_remove::<Selected>(),
        ));
        // Update the view_data so we can keep track of which zindex is the topmost
        view_data.top_z += 0.0001;
        
        // NAME LABEL
        // This is the text label that will be attached to the node.
        // It will be spawned as a child of the node entity.
        
        let name_label = commands.spawn(
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
                    translation: Vec3::new(radius + 14.0, 0.0, 100.1),
                    ..default()
                },
                text_anchor: bevy::sprite::Anchor::CenterLeft,
                ..default()
            }
        ).id();
        
        commands.entity(ev.entity).push_children(&[name_label]);
        
        // OUTLINE
        // ----------------------------------------------------------------
        // This is the hoverable, interactable outline from which edges are created.
        // The shape of it is determined by the users view settings as well as the
        // type of node it outlines. 
        
        let outline_width = 10.0;
        let outline_width_hovered = 12.0;
        
        let outline_shape = shapes::Circle {
            radius: radius + outline_width / 2.,
            center: Vec2::from((0.0, 0.0)),
        };
        //let outline_shape_hovered = shapes::Circle {
        //     radius: radius + outline_width_hovered / 2.,
        //     center: Vec2::from((0.0, 0.0)),
        // };
        let outline_path = GeometryBuilder::build_as(&outline_shape);
        //let outline_path_hovered = GeometryBuilder::build_as(&outline_shape_hovered);
        
        let node_outline = commands.spawn((
            ShapeBundle {
                path: outline_path,
                ..default()
            },
            Stroke::new(
                crate::settings::theme::OUTLINE_BASE_COLOR, 5.0
            ),
            NodeOutline,
            
            PickableBundle::default(),
            RaycastPickable::default(),

            On::<Pointer<Over>>::target_component_mut::<Stroke>(move |_over, stroke| {
                stroke.color = crate::settings::theme::OUTLINE_HOVER_COLOR;
                stroke.options = StrokeOptions::default().with_line_width(outline_width_hovered);
            }),
            
            On::<Pointer<Out>>::target_component_mut::<Stroke>(move |_out, stroke| {
                stroke.color = crate::settings::theme::OUTLINE_BASE_COLOR;
                stroke.options = StrokeOptions::default().with_line_width(outline_width);
            }),
        )).id();
        
        commands.entity(ev.entity).push_children(&[node_outline]);
        
    }
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


                    