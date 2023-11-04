use bevy::{prelude::*, sprite::MaterialMesh2dBundle, text::Text2dBounds};
use bevy_mod_picking::prelude::*;
use bevy_prototype_lyon::{shapes, prelude::{GeometryBuilder, ShapeBundle, Fill, Stroke, StrokeOptions}, entity};
use rand::Rng;

use crate::{
    graph::{nodes::GraphNode, graph_cam::ViewData, context::Selected}, 
    events::nodes::{MoveNodesEvent, NodeClickEvent, NodePressedEvent, NodeHoverEvent}
};

#[derive(Component)]
pub struct NodeOutline;

pub fn add_node_ui(
    commands: &mut Commands,
    entity: Entity,
    path: String,
    name: String,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    view_data: &mut ResMut<ViewData>,
){
    println!("Adding node UI");

    let full_path = format!("{}/{}", path, name);
    
    // Positions will be random for now
    let mut rng = rand::thread_rng();
    
    //let node_ui = commands.spawn(
    commands.entity(entity).insert((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(25.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::rgb(0.3, 0.0, 0.0))),
            transform: Transform::from_translation(Vec3::new(
                rng.gen_range(-10.0..10.0),
                rng.gen_range(-10.0..10.0),
                view_data.top_z,
            )),
            ..default()
        },
        PickableBundle::default(),
        RaycastPickTarget::default(),
        
        On::<Pointer<Drag>>::send_event::<MoveNodesEvent>(),
        On::<Pointer<Click>>::send_event::<NodeClickEvent>(),
        On::<Pointer<Down>>::send_event::<NodePressedEvent>(),
        On::<Pointer<Over>>::send_event::<NodeHoverEvent>(),
        // On::<Pointer<Drag>>::run(move_node_selection),
        On::<Pointer<DragStart>>::target_insert(Selected),
        On::<Pointer<DragEnd>>::target_remove::<Selected>(),
        On::<Pointer<Deselect>>::target_remove::<Selected>(),
    ));
    
    // Create the name label for the node.
    let name_label = commands.spawn(Text2dBundle {
        text: Text {
            sections: vec![TextSection::new(
                name,
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
            translation: Vec3::new(35.0, 0.0, 100.1),
            ..default()
        },
        text_anchor: bevy::sprite::Anchor::CenterLeft,
        ..default()
    }).id();
    
    commands.entity(entity).push_children(&[name_label]);
    
    let outline_shape = shapes::Circle {
        radius: 30.0,
        center: Vec2::from((0.0, 0.0)),
    };

    let node_outline = commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&outline_shape),
                ..default()
            },
            Stroke::new(Color::ORANGE, 10.0),
            NodeOutline,

            PickableBundle::default(),
            RaycastPickTarget::default(),
    )).id();
    
    commands.entity(entity).push_children(&[node_outline]);
        
    // Commented out for now. 
    // Spawning nodes is generic until I figure out
    // how to handle different types of nodes. 
    
    // parent.spawn((SpriteBundle {
    //     texture: image_handle,
    //     sprite: Sprite {
    //         anchor: bevy::sprite::Anchor::TopLeft,
    //         ..default()
    //     },
    
    //     transform: Transform {
    //         translation: Vec3::new(0.0, -30.0, 0.1),
    //         scale: Vec3::new(1.0, 1.0, 0.05),
    //         ..default()
    //     },
    //     ..default()
    // },
    // PickableBundle {
    //     pickable: Pickable {
    //         should_block_lower: false,
    //         ..default()
    //     },
    //     ..default()
    // }
    // ));        
    
    
    // Update the view_data so we can keep track of which zindex is the topmost
    view_data.top_z += 0.0001;
    
}

pub fn outlines_pulse(
    time: Res<Time>,
    mut outlines: Query<&mut Stroke, With<NodeOutline>>,
){
    let line_width = 5.0 + (time.elapsed_seconds() * 4.0).sin() * 5.0;
    for mut outline in outlines.iter_mut(){
        outline.options = StrokeOptions::default().with_line_width(line_width);
    }
}


                    
pub fn handle_outline_hover(
    // ev_spawn: EventReader<Node>,
    nodes: Query<&Transform, With<GraphNode>>,
    
    mut commands: Commands,
){
    for pos in nodes.iter(){

    }
}