//Code pertaining to the graph nodes

use bevy::{prelude::*, input::keyboard::KeyboardInput};

use super::{graph_cam, context::{PathsToEntitiesIndex, ToBeDespawned, Selected}, node_types::NodeTypes};

use crate::{events::nodes::*, ui::nodes::{NodeOutline, GraphViewNode}, input::pointer::InputData};
use crate::ui::nodes::add_node_ui;

pub struct NodesPlugin;

impl Plugin for NodesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(PreUpdate, handle_node_click)
            .add_systems(PreUpdate, handle_node_press)
            .add_systems(PreUpdate, handle_node_hover)

            .add_systems(PostUpdate, despawn_nodes)
        ;
    }
}

// ----------------------------------------------------------------
// Component definitions

// A component to store the data of a NODE
// The path and name of the node is something that all node types have in common
#[derive(Component)]
pub struct GraphDataNode {
    pub path: String,
    pub name: String,
}

#[derive(Component)]
pub struct NodeType {
    pub ntype: NodeTypes,
}

// A component to store the edge relationships of a node
// Stores a vec to the edge entities
#[derive(Component)]
pub struct GraphNodeEdges {
    pub edges: Vec<Entity>,
}

#[derive(Component)]
pub struct ContextRoot;

#[derive(Component)]
pub struct PinnedToPosition;

#[derive(Component)]
pub struct PinnedToPresence;

#[derive(Component)]
pub struct PinnedToUi;

// ? Should there be a:
// DataNode Bundle
// The basic component bundle of every node. This is shared between all of them,
// regardless of type.

// ----------------------------------------------------------------
// Interaction systems


fn handle_node_click(
    mut event: EventReader<NodeClickEvent>,
    mut keys: EventReader<KeyboardInput>,

    mut commands: Commands,
    mut input_data: ResMut<InputData>,

    nodes: Query<(Entity, &GraphDataNode)>,
    selection: Query<Entity, (With<GraphViewNode>, With<Selected>)>,
    outlines: Query<&Parent, With<NodeOutline>>,
){
    if event.is_empty(){
        return
    }

    if !keys.read().any(
        |k| k.key_code == Some(KeyCode::ShiftLeft) 
        || k.key_code == Some(KeyCode::ShiftRight)
    ) //&& !mouse.pressed(MouseButton::Right) 
    {
        println!("Clearing selection");
        for node in selection.iter() {
            commands.entity(node).remove::<Selected>();
        }
    }

    // TODO: Handle multiple events
    match event.read().next().unwrap().target {
        None => {
            //println!("No event");
            input_data.latest_click_entity = None;
        }
        Some(target) => {
            //println!("Event: {:?}", target);
            
            match nodes.get(target){
                Ok(node) => {
                    let target_path = node.1.path.clone();
                    input_data.latest_click_entity = Some(target_path.clone());
                    commands.entity(node.0).insert(Selected);
                    //println!("Clicking path: {}", target_path);
                },
                Err(_) => {
                    //println!("No node found");
                }
            }

            match outlines.get(target){
                Ok(outline) => {
                    let outline_path = nodes.get(outline.get()).unwrap().1.path.clone();
                    input_data.latest_click_entity = Some(outline_path.clone());
                    //println!("Clicking outline: {}", outline_path);
                },
                Err(_) => {
                    //println!("No outline found");
                }
            }
        },
    }
}

fn handle_node_press(
    mut event: EventReader<NodePressedEvent>,
    mut input_data: ResMut<InputData>,
    nodes: Query<&GraphDataNode>,
    outlines: Query<&Parent, With<NodeOutline>>,
){
    if event.is_empty() {
        return
    }
    match event.read().next().unwrap().target {
        None => {
            //println!("No event");
            input_data.latest_press_entity = None;
        }
        Some(target) => {
            //println!("Event: {:?}", target);
            
            match nodes.get(target){
                Ok(node) => {
                    let target_path = node.path.clone();
                    input_data.latest_press_entity = Some(target_path.clone());
                    input_data.press_is_outline = false;
                    println!("Pressing path: {}", input_data.latest_press_entity.clone().unwrap());
                },
                Err(_) => {
                    //println!("No node found for press");
                }
            }

            match outlines.get(target){
                Ok(outline) => {
                    let outline_path = nodes.get(outline.get()).unwrap().path.clone();
                    input_data.latest_press_entity = Some(outline_path.clone());
                    input_data.press_is_outline = true;
                    //println!("Pressing outline: {}", outline_path);
                },
                Err(_) => {
                    //println!("No outline found");
                }
            }
        },
    }
}


fn handle_node_hover(
    mut event: EventReader<NodeHoverEvent>,
    mut input_data: ResMut<InputData>,
    nodes: Query<&GraphDataNode>,
    outlines: Query<&Parent, With<NodeOutline>>,
){
    if event.is_empty() {
        return
    }
    match event.read().next().unwrap().target {
        None => {
            //println!("No event");
            input_data.latest_hover_entity = None;
        }
        Some(target) => {
            //println!("Event: {:?}", target);
            
            match nodes.get(target){
                Ok(node) => {
                    let target_path = node.path.clone();
                    input_data.latest_hover_entity = Some(target_path.clone());
                    //println!("Hovering over path: {}", target_path);
                },
                Err(_) => {
                    //println!("No node found");
                }
            }

            match outlines.get(target){
                Ok(outline) => {
                    let outline_path = nodes.get(outline.get()).unwrap().path.clone();
                    input_data.latest_hover_entity = Some(outline_path.clone());
                    //println!("Hovering over outline: {}", outline_path);
                },
                Err(_) => {
                    //println!("No outline found");
                }
            }
        },
    }
}


// ----------------------------------------------------------------
// Spawning and despawning systems


pub fn spawn_node (
    event: &mut EventWriter<NodeSpawnedEvent>,

    commands: &mut Commands,
    path: &String,
    name: &String,
    ntype: NodeTypes,
    position: Vec2, // For the viewnodes

    pe_index: &mut ResMut<PathsToEntitiesIndex>,
) -> bevy::prelude::Entity {
    let full_path = format!("{}/{}", path, name);


    let node_entity = commands.spawn((
        GraphDataNode {
            path: full_path.clone(),
            name: name.clone()
        },
    )).id();

    event.send(NodeSpawnedEvent {
        entity: node_entity,
        path: path.to_string(),
        name: name.to_string(),
        ntype,
        position: position,
    });

    // Update the PathsToEntitiesIndex
    pe_index.0.insert(full_path, node_entity);

    // Return the node entity
    node_entity
}

fn despawn_nodes(
    mut commands: Commands,
    mut nodes: Query<(Entity, &GraphDataNode), With<ToBeDespawned>>,
    mut pe_index: ResMut<PathsToEntitiesIndex>,
){
    for (entity, node) in nodes.iter_mut() {
        commands.entity(entity).despawn_recursive();
        pe_index.0.remove(&node.path);
    }
}