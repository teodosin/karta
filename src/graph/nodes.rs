//Code pertaining to the graph nodes

use bevy::{prelude::*, sprite::MaterialMesh2dBundle, text::Text2dBounds};
use bevy_mod_picking::prelude::*;
use rand::Rng;

use super::{graph_cam, context::{Selected, PathsToEntitiesIndex, ToBeDespawned}};

// Component definitions

// A component to store the data of a NODE
#[derive(Component)]
pub struct GraphNode {
    pub path: String,
}

// A component to store the edge relationships of a node
// Stores a vec to the edge entities
#[derive(Component)]
pub struct GraphNodeEdges {
    pub edges: Vec<Entity>,
}

// ----------------------------------------------------------------
// Interaction systems

// The main input event for nodes. 
#[derive(Event)]
pub struct NodeClickEvent {
    pub target: Option<Entity>,
    pub button: MouseButton,
}

// Implementation required by bevy_mod_picking
impl From<ListenerInput<Pointer<Click>>> for NodeClickEvent {
    fn from(event: ListenerInput<Pointer<Click>>) -> Self {
        NodeClickEvent {
            target: Some(event.target),
            button: match event.button {
                // It's interesting that bevy mod picking and bevy input 
                // use different enums for mouse buttons. Asked the crate
                // author on Discord about it. 
                PointerButton::Primary => MouseButton::Left,
                PointerButton::Secondary => MouseButton::Right,
                PointerButton::Middle => MouseButton::Middle,
                //_ => MouseButton::Other,
            }
        }
    }
}

#[derive(Event)]
pub struct NodePressedEvent {
    pub target: Option<Entity>,
    pub button: MouseButton,

}

impl From<ListenerInput<Pointer<Down>>> for NodePressedEvent {
    fn from(event: ListenerInput<Pointer<Down>>) -> Self {
        NodePressedEvent {
            target: Some(event.target),
            button: match event.button {
                PointerButton::Primary => MouseButton::Left,
                PointerButton::Secondary => MouseButton::Right,
                PointerButton::Middle => MouseButton::Middle,
            }
        }
    }
}

#[derive(Event)]
pub struct NodeHoverEvent {
    pub target: Option<Entity>,
}

// Implementation required by bevy_mod_picking
impl From<ListenerInput<Pointer<Over>>> for NodeHoverEvent {
    fn from(event: ListenerInput<Pointer<Over>>) -> Self {
        NodeHoverEvent {
            target: Some(event.target),
        }
    }
}

#[derive(Event)]
pub struct MoveNodesEvent;

// Implementation required by bevy_mod_picking
impl From<ListenerInput<Pointer<Drag>>> for MoveNodesEvent {
    fn from(_event: ListenerInput<Pointer<Drag>>) -> Self {
        MoveNodesEvent
    }
}

pub fn handle_node_click(
    mouse: Res<Input<MouseButton>>,
    mut event: EventReader<NodeClickEvent>,
    mut input_data: ResMut<graph_cam::InputData>,
    nodes: Query<&GraphNode>,
){
    if event.is_empty(){
        return
    }

    match event.iter().next().unwrap().target {
        None => {
            println!("No event");
            input_data.latest_click_entity = None;
        }
        Some(target) => {
            println!("Event: {:?}", target);
            let target_path = nodes.get(target).unwrap().path.clone();
        
            input_data.latest_click_entity = Some(target_path.clone());
            println!("Target path: {}", target_path);
        },
    }
}

pub fn handle_node_press(
    mouse: Res<Input<MouseButton>>,
    mut event: EventReader<NodePressedEvent>,
    mut input_data: ResMut<graph_cam::InputData>,
    nodes: Query<&GraphNode>,
){
    if event.is_empty() {
        return
    }
    match event.iter().next().unwrap().target {
        None => {
            println!("No event");
            input_data.latest_press_entity = None;
        }
        Some(target) => {
            println!("Event: {:?}", target);
            let target_path = nodes.get(target).unwrap().path.clone();
        
            input_data.latest_press_entity = Some(target_path.clone());
            println!("Target path: {}", target_path);
        },
    }
}


pub fn handle_node_hover(
mut event: EventReader<NodeHoverEvent>,
mut input_data: ResMut<graph_cam::InputData>,
nodes: Query<&GraphNode>,
){
    if event.is_empty() {
        return
    }
    match event.iter().next().unwrap().target {
        None => {
            println!("No event");
            input_data.latest_hover_entity = None;
        }
        Some(target) => {
            println!("Event: {:?}", target);
            let target_path = nodes.get(target).unwrap().path.clone();
        
            input_data.latest_hover_entity = Some(target_path.clone());
            println!("Hovering over path: {}", target_path);
        },
    }
}


// ----------------------------------------------------------------
// Spawning and despawning systems

pub fn spawn_node (
    commands: &mut Commands,
    path: &String,
    name: &String,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    view_data: &mut ResMut<graph_cam::ViewData>,
    pe_index: &mut ResMut<PathsToEntitiesIndex>,
) -> bevy::prelude::Entity {
    let full_path = format!("{}/{}", path, name);

    // Positions will be random for now
    let mut rng = rand::thread_rng();

    let node = commands.spawn((
        GraphNode {
            path: full_path.clone()
        },
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(25.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::rgb(0.3, 0.0, 0.0))),
            transform: Transform::from_translation(Vec3::new(
                rng.gen_range(-600.0..600.0),
                rng.gen_range(-400.0..400.0),
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
    )).id();
        
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
            translation: Vec3::new(30.0, 0.0, 0.1),
            ..default()
        },
        text_anchor: bevy::sprite::Anchor::CenterLeft,
        ..default()
    }).id();

    commands.entity(node).push_children(&[name_label]);
        
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
    view_data.top_z += 1.;

    // Update the PathsToEntitiesIndex
    pe_index.0.insert(full_path, node);

    // Return the node entity
    node
    
}

pub fn despawn_nodes(
    mut commands: Commands,
    mut nodes: Query<(Entity, &GraphNode), With<ToBeDespawned>>,
    mut pe_index: ResMut<PathsToEntitiesIndex>,
){
    for (entity, node) in nodes.iter_mut() {
        commands.entity(entity).despawn_recursive();
        pe_index.0.remove(&node.path);
    }
}