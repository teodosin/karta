//Code pertaining to the graph nodes

use std::{path::PathBuf, ffi::OsString};

use bevy::{prelude::*, input::keyboard::KeyboardInput};
use bevy_mod_picking::picking_core::PickSet;

use super::{context::{PathsToEntitiesIndex, ToBeDespawned, Selected}, node_types::{NodeTypes, NodeData, type_to_data}};

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
    pub path: PathBuf,
    pub name: OsString,
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
#[derive(Component)]
pub struct GraphNodeEdges {
    pub edges: Vec<Entity>,
}

impl Default for GraphNodeEdges {
    fn default() -> Self {
        GraphNodeEdges {
            edges: Vec::new(),
        }
    }
}

impl GraphNodeEdges {
    pub fn add_edge(&mut self, edge: Entity) {
        self.edges.push(edge);
    }

    pub fn remove_edge(&mut self, edge: Entity) {
        self.edges.retain(|&e| e != edge);
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
    name: OsString,
    ntype: NodeTypes,
    position: Vec2, // For the viewnodes

    pe_index: &mut ResMut<PathsToEntitiesIndex>,
) -> bevy::prelude::Entity {

    let full_path = path.join(&name);


    let data = type_to_data(ntype);

    let node_entity = commands.spawn((
        GraphDataNode {
            path: full_path.clone(),
            name: name.clone(),
            ntype,
            data: None,
        },
        GraphNodeEdges::default()
    )).id();

    event.send(NodeSpawnedEvent {
        entity: node_entity,
        path: path,
        name: name,
        ntype,
        data,
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