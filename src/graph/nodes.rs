//Code pertaining to the graph nodes

use bevy::prelude::*;

use super::{graph_cam, context::{PathsToEntitiesIndex, ToBeDespawned}};

use crate::{events::nodes::*, ui::nodes::NodeOutline, input::pointer::InputData};
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
#[derive(Component)]
pub struct GraphNode {
    pub path: String,
    pub name: String,
}

// A component to store the edge relationships of a node
// Stores a vec to the edge entities
#[derive(Component)]
pub struct GraphNodeEdges {
    pub edges: Vec<Entity>,
}

// ----------------------------------------------------------------
// Interaction systems


fn handle_node_click(
    mouse: Res<Input<MouseButton>>,
    mut event: EventReader<NodeClickEvent>,
    mut input_data: ResMut<InputData>,
    nodes: Query<&GraphNode>,
    outlines: Query<&Parent, With<NodeOutline>>,
){
    if event.is_empty(){
        return
    }

    match event.iter().next().unwrap().target {
        None => {
            //println!("No event");
            input_data.latest_click_entity = None;
        }
        Some(target) => {
            //println!("Event: {:?}", target);
            
            match nodes.get(target){
                Ok(node) => {
                    let target_path = node.path.clone();
                    input_data.latest_click_entity = Some(target_path.clone());
                    //println!("Clicking path: {}", target_path);
                },
                Err(_) => {
                    //println!("No node found");
                }
            }

            match outlines.get(target){
                Ok(outline) => {
                    let outline_path = nodes.get(outline.get()).unwrap().path.clone();
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
    nodes: Query<&GraphNode>,
    outlines: Query<&Parent, With<NodeOutline>>,
){
    if event.is_empty() {
        return
    }
    match event.iter().next().unwrap().target {
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
nodes: Query<&GraphNode>,
outlines: Query<&Parent, With<NodeOutline>>,
){
    if event.is_empty() {
        return
    }
    match event.iter().next().unwrap().target {
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
    mut commands: &mut Commands,
    path: &String,
    name: &String,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<ColorMaterial>>,
    mut view_data: &mut ResMut<graph_cam::ViewData>,
    mut pe_index: &mut ResMut<PathsToEntitiesIndex>,
) -> bevy::prelude::Entity {
    let full_path = format!("{}/{}", path, name);


    let node_entity = commands.spawn((
        GraphNode {
            path: full_path.clone(),
            name: name.clone()
        },
    )).id();

    add_node_ui(
        &mut commands,
        node_entity,
        full_path.clone(),
        name.to_string(),
        &mut meshes,
        &mut materials,
        &mut view_data,
    );

    // Update the PathsToEntitiesIndex
    pe_index.0.insert(full_path, node_entity);

    // Return the node entity
    node_entity
}

fn despawn_nodes(
    mut commands: Commands,
    mut nodes: Query<(Entity, &GraphNode), With<ToBeDespawned>>,
    mut pe_index: ResMut<PathsToEntitiesIndex>,
){
    for (entity, node) in nodes.iter_mut() {
        commands.entity(entity).despawn_recursive();
        pe_index.0.remove(&node.path);
    }
}