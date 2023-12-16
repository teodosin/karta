//Code pertaining to the graph nodes

use std::{path::PathBuf, ffi::OsString};

use bevy::{prelude::*, input::keyboard::KeyboardInput, utils::HashMap};
use bevy_mod_picking::picking_core::PickSet;

use super::{context::{PathsToEntitiesIndex, ToBeDespawned, Selected, CurrentContext}, node_types::{NodeTypes, NodeData, type_to_data}};

use crate::{events::nodes::*, ui::nodes::{NodeOutline, GraphViewNode}, input::pointer::{InputData, update_cursor_info}};

pub struct NodesPlugin;

impl Plugin for NodesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(PreUpdate, (
                handle_node_click,
                handle_node_press,
                handle_node_hover,
            )
            .before(update_cursor_info)
            .after(PickSet::Last))

            .add_systems(PostUpdate, despawn_nodes
                // .run_if(resource_changed::<CurrentContext>())
            )
        ;
    }
}

// ----------------------------------------------------------------
// Component definitions

/// A component to store the universal data of a node. 
/// The path and name of the node is something that all node types have in common.
/// The name is the file name.
#[derive(Component)]
pub struct GraphDataNode {
    pub path: PathBuf,
    pub ntype: NodeTypes,
    pub data: Option<Box<dyn NodeData>>,
}

impl GraphDataNode {
    pub fn _get_data_type(&self) -> String {
        let ntype = match &self.data {
            None => String::from("No data"),
            Some(data) => data.get_data_type(),
        };
        ntype
    }

    pub fn _get_data(&self, world: &World) -> Option<Box<dyn NodeData>> {
        let data = match self.data {
            None => {
                //println!("No data");
                return None;
            },
            Some(ref data) => Some(data.get_data(world, &self.path)),
        };
        data
    }
}

// A component to store the edge relationships of a node
// Stores a vec to the edge entities
#[derive(Component, Clone, Debug, Default)]
pub struct GraphNodeEdges {
    // The key is a reference to the other node, the value is the edge entity
    // TODO: Entities aren't stable across instances, so getting the correct edge entity
    // is not guaranteed. No systems need this data yet, but it will be a problem in the future.
    pub edges: HashMap<PathBuf, Entity>,
}

impl GraphNodeEdges {
    pub fn insert_edge(&mut self, node_path: PathBuf, edge: Entity) {
        println!("Adding edge: {:?} to node {:?}", edge, node_path);
        self.edges.insert(node_path, edge);
    }

    pub fn remove_edge(&mut self, edge: Entity) {
        println!("Removing edge: {:?}", edge);
        self.edges.retain(|_, v| *v != edge);
    }
}


#[derive(Component)]
pub struct ContextRoot;

#[derive(Component)]
pub struct PinnedToPosition;

#[derive(Component)]
pub struct PinnedToPresence;

#[derive(Component)]
pub struct PinnedToUi;

// Marker Component for nodes that are only visitors to the current context and should not be serialized
#[derive(Component)]
pub struct Visitor;


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
                    let (entity, data) = node;
                    let target_path = data.path.clone();
                    input_data.latest_click_entity = Some(target_path.clone());
                    commands.entity(entity).insert(Selected);
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
                    println!("Pressing path: {}", input_data.latest_press_entity.clone().unwrap().display());
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

// NOTE: This function can't be used in the CreateNodeAction directly, but
// the two must be kept in sync. 
// TODO: Address this limitation. 
pub fn spawn_node (
    event: &mut EventWriter<NodeSpawnedEvent>,
    commands: &mut Commands,

    path: PathBuf,
    ntype: NodeTypes,

    root_position: Vec2, // For the viewnodes
    rel_target_position: Option<Vec2>, // For the viewnodes

    pinned_to_position: bool,

    pe_index: &mut ResMut<PathsToEntitiesIndex>,

) -> bevy::prelude::Entity {
    if pe_index.0.contains_key(&path) {
        println!("Node already exists");
        return pe_index.0.get(&path).unwrap().clone();
    }

    let data = type_to_data(ntype);

    let node_entity = commands.spawn((
        GraphDataNode {
            path: path.clone(),
            ntype,
            data: None,
        },
        GraphNodeEdges::default()
    )).id();

    event.send(NodeSpawnedEvent {
        entity: node_entity,
        path: path.clone(),
        ntype,
        data,
        root_position,
        rel_target_position,
        pinned_to_position,
    });

    // Update the PathsToEntitiesIndex
    pe_index.0.insert(path, node_entity);

    // Return the node entity
    node_entity
}

fn despawn_nodes(
    mut commands: Commands,
    mut nodes: Query<(Entity, &GraphDataNode), With<ToBeDespawned>>,
    mut pe_index: ResMut<PathsToEntitiesIndex>,
){
    let len = nodes.iter_mut().count();
    if len == 0 {
        return
    }
    println!("About to despawn {} nodes", nodes.iter_mut().count());
    for (entity, node) in nodes.iter_mut() {
        commands.entity(entity).despawn_recursive();
        pe_index.0.remove(&node.path);
    }
}