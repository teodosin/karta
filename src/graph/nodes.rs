//Code pertaining to the graph nodes

use bevy::prelude::*;

use super::{graph_cam, context::{PathsToEntitiesIndex, ToBeDespawned}};

use crate::events::nodes::*;
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

fn handle_node_press(
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


fn handle_node_hover(
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
    mut commands: &mut Commands,
    path: &String,
    name: &String,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<ColorMaterial>>,
    mut view_data: &mut ResMut<graph_cam::ViewData>,
    mut pe_index: &mut ResMut<PathsToEntitiesIndex>,
) -> bevy::prelude::Entity {
    let full_path = format!("{}/{}", path, name);

    // Positions will be random for now
    let mut rng = rand::thread_rng();

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