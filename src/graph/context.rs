/// Main file for the Context plugin
/// The Context manages the node graph 
/// 

use bevy::{prelude::*, utils::HashMap};
use std::fs;

use crate::{graph::graph_cam, modes::edges::create_edge, vault::KartaVault};

use super::{nodes::*, edges::*};

pub struct ContextPlugin;

impl Plugin for ContextPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(PathsToEntitiesIndex(HashMap::new()))
            .insert_resource(CurrentContext::new())

            .add_event::<NodeClickEvent>()
            .add_event::<NodePressedEvent>()
            .add_event::<NodeHoverEvent>()
            .add_event::<MoveNodesEvent>()

            .add_systems(Startup, initial_context)

            .add_systems(PreUpdate, handle_node_click)
            .add_systems(PreUpdate, handle_node_press)
            .add_systems(PreUpdate, handle_node_hover)

            .add_systems(Update, update_context
                .run_if(resource_changed::<CurrentContext>())
            )

            .add_systems(PostUpdate, despawn_nodes.after(update_context))
            .add_systems(PostUpdate, draw_edges)
            
            .add_systems(Last, despawn_edges
                .run_if(resource_changed::<CurrentContext>())
            )
        ;
    }
}

#[derive(Resource, Debug)]
pub struct PathsToEntitiesIndex(
    pub HashMap<String, Entity>,
);

// The resource that stores the current context path
#[derive(Resource, Debug)]
pub struct CurrentContext{
    pub current_context: String,
}

// A marker component for selected graph entities
#[derive(Component, Clone)]
pub struct Selected;

#[derive(Component)]
pub struct ToBeDespawned;

impl CurrentContext {
    fn new() -> Self {
        CurrentContext {
            current_context: "home/viktor/Pictures".to_string(),
        }
    }

    pub fn set_current_context(&mut self, path: String) {
        self.current_context = path;
    }

    pub fn get_current_context_path(&self) -> String {
        format!("/{}", self.current_context)
    }
}



fn initial_context(
    mut event: EventWriter<NodeClickEvent>,
){
    event.send(NodeClickEvent {
        target: None,
        button: MouseButton::Left,
    });
}

// Big monolith function
pub fn update_context(
    input_data: Res<graph_cam::InputData>,

    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,

    vault: Res<KartaVault>,
    context: Res<CurrentContext>,

    mut view_data: ResMut<graph_cam::ViewData>,
    mut pe_index: ResMut<PathsToEntitiesIndex>,

    mut nodes: Query<(Entity, &GraphNode)>,
) {
    
    // Handle the path to the desired context
    let path: String = input_data.latest_click_entity.clone()
    .unwrap_or(context.get_current_context_path());
    // Also return if the target path is already the current context


    println!("Path: {}", path);
    let entries = fs::read_dir(&path);

    // Get all file and folder names in 
    let entries = match entries {
        Ok(entries) => entries,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };

    // Get all files
    let mut file_names: Vec<String> = entries
    // Ignore the vault folder!!!
    .filter(|entry| {
        let path = entry.as_ref().unwrap().path().to_str().unwrap().to_string();
        println!("Path: {}", path);
        path != vault.get_vault_path()
    })
    // Carry on with everything else
    .filter_map(|entry| {
        let path = entry.ok()?.path();
        if path.is_file() {
            path.file_name()?.to_str().map(|s| s.to_owned())
        } else {
            path.file_name()?.to_str().map(|s| s.to_owned())
        }
    })
    .collect();

    file_names.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

    for file in file_names.iter() {
        println!("File: {}", file);
    }

    // Iterate through existing nodes and mark them for deletion
    for (entity, _node) in nodes.iter_mut() {
        commands.entity(entity).insert(ToBeDespawned);
    }

    // Spawn the context root if it doesn't exist
    let root_node = match pe_index.0.get(&path) {
        Some(entity) => {
            println!("Root node already exists");
            commands.entity(*entity).remove::<ToBeDespawned>();
            *entity
        },
        None => {
            println!("Root node doesn't exist, spawning");
            let root_name = path.split("/").last().unwrap().to_string();
            let root_path = path.replace(&root_name, "");
            let root_path = &root_path[0..&root_path.len()-1].to_string();
            println!("Root Path: {}, Root Name: {}", root_path, root_name);
            spawn_node(
                &mut commands, 
                &root_path, 
                &root_name,
                &mut meshes, 
                &mut materials, 
                &mut view_data,
                &mut pe_index,
            )
        }
    };

    // Don't despawn the parent of the root
    let root_parent_path = path
        .replace(&path.split("/")
        .last()
        .unwrap(), "");
    let root_parent_path = &root_parent_path[0..&root_parent_path.len()-1].to_string();
    let root_parent = pe_index.0.get(root_parent_path);
    println!("Root parent: {:?}", root_parent_path);
    match root_parent {
        Some(entity) => {
            commands.entity(*entity).remove::<ToBeDespawned>();
        },
        None => {
            println!("Root parent doesn't exist");
        }
    }
    
    file_names.iter().for_each(|name| {

        // Check if the item already exists
        let full_path = format!("{}/{}", path, name);
        let item_exists = pe_index.0.get(&full_path).is_some();
        if item_exists {
                println!("Item already exists: {}", full_path);
                // Remove despawn component
                commands.entity(pe_index.0.get(&full_path).unwrap().clone()).remove::<ToBeDespawned>();
                return
        }

        // Spawn a node for each item
        let node = spawn_node(
            &mut commands,
            &path,
            name,
            &mut meshes,
            &mut materials,
            &mut view_data,
            &mut pe_index,
        );

        // Spawn an edge from the root node to each item
        create_edge(&root_node, &node, &mut commands);
    });

    // Print pe_index to see what the hell is going on
    for (path, entity) in pe_index.0.iter() {
        println!("Path: {}, Entity: {:?}", path, entity);
    };
}

// Collapse and expand functions

// Similar to the spawn functions, but manages aliases also 
// So that when a node group is collapsed, it is replaced by its alias edge
// The edge that pointed to that node now points to the alias edge

// If a node group is expanded, the alias edge is replaced by the node group
// and their relevant edges.
// If an individual node is expanded and its file format is supported,
// its contents and their relevant edges are spawned around it (or in it)

