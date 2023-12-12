/// Main file for the Context plugin
/// The Context manages what is in the node graph. 
/// Responsible for keeping track of what is spawned and despawned.

use bevy::{prelude::*, utils::HashMap};
use bevy_mod_picking::prelude::PointerButton;
use std::{fs, path::PathBuf, ffi::OsString};

use crate::{
    graph::{graph_cam, edges::{create_edge, EdgeTypes}, node_types::get_type_from_path}, vault::CurrentVault, 
    events::{nodes::{NodeClickEvent, NodeSpawnedEvent}, edges::EdgeSpawnedEvent},
};

use super::{nodes::*, edges::{GraphEdge, EdgeType}};

pub struct ContextPlugin;

impl Plugin for ContextPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(PathsToEntitiesIndex(HashMap::new()))
            .insert_resource(CurrentContext::new())

            .add_systems(Startup, initial_context)

            .add_systems(Last, update_context
                .run_if(resource_changed::<CurrentContext>())
            )
        ;
    }
}

#[derive(Resource, Debug)]
pub struct PathsToEntitiesIndex(
    pub HashMap<PathBuf, Entity>,
);

// The resource that stores the current context path
#[derive(Resource, Debug)]
pub struct CurrentContext{
    pub cxt: Option<KartaContext>,
    // A context can be any file or node defined inside some context. 
    // Any node that is created which is not a file in itself, must still
    // be stored somewhere. That place is the closest parent folder. 
}

impl CurrentContext {
    fn new() -> Self {
        CurrentContext {
            cxt: None,
        }
    }

    pub fn set_current_context(&mut self, path: PathBuf) {
        let path = path.canonicalize().unwrap();
        let mut dir_path = path.clone();
        dir_path.pop();
        if dir_path.exists() {
            if dir_path.is_dir() {
                println!("Path is a directory: {}", dir_path.display());
            } else {
                println!("Path is a file: {}", dir_path.display());
                dir_path.pop();

                if dir_path.exists() {
                    if dir_path.is_dir() {
                        println!("Path is a directory: {}", dir_path.display());
                    } else {
                        println!("VIRTUAL NODE TOO DEEP: {}", dir_path.display());
                        return
                    }
                } else {
                    println!("Path doesn't exist: {}", dir_path.display());
                    return
                }
            }
        } else {
            println!("Path doesn't exist: {}", path.display());
            return
        }
        self.cxt = Some(KartaContext {
            current_context: path,
            current_context_directory: dir_path.to_path_buf(),
        });
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct KartaContext {
    // Path to the current context, including the file or dir name
    pub current_context: PathBuf,
    // Path to the current context's parent directory
    pub current_context_directory: PathBuf,
}

impl KartaContext {
    pub fn get_current_context_path(&self) -> PathBuf {
        self.current_context.clone()
    }
}
// A marker component for selected graph entities
#[derive(Component, Clone)]
pub struct Selected;

#[derive(Component)]
pub struct ToBeDespawned;







fn initial_context(
    mut event: EventWriter<NodeClickEvent>,
){
    event.send(NodeClickEvent {
        target: None,
        button: PointerButton::Primary,
    });
}

// Big monolith function
// --------------------------------------------------------------------------------
pub fn update_context(
    mut node_event: EventWriter<NodeSpawnedEvent>,
    mut edge_event: EventWriter<EdgeSpawnedEvent>,
    
    mut commands: Commands,

    vault: Res<CurrentVault>,
    context: Res<CurrentContext>,

    mut view_data: ResMut<graph_cam::ViewData>,
    mut pe_index: ResMut<PathsToEntitiesIndex>,

    mut nodes: Query<(Entity, &GraphDataNode, &Transform)>,
    edges: Query<(&GraphEdge, &EdgeType)>,
    root: Query<Entity, With<ContextRoot>>,
) {
    let vault = &vault.vault;
    let vault = match vault {
        Some(vault) => vault,
        None => {
            println!("No vault set");
            return
        }
    };

    // Handle the path to the desired context
    let path = match context.cxt {
        Some(ref cxt) => cxt.get_current_context_path(),
        None => {
            println!("No context set");
            return
        }
    };
    // Also return if the target path is already the current context

    // Iterate through existing nodes and mark them for deletion
    for (entity, _node, _pos) in nodes.iter_mut() {
        commands.entity(entity).insert(ToBeDespawned);
    }

    let root_position: Vec2;

    // Spawn the context root if it doesn't exist
    let root_node = match pe_index.0.get(&path) {
        Some(entity) => {
            println!("Root node already exists");
            // Get position of root node
            root_position = nodes.get(*entity).unwrap().2.translation.truncate();
            commands.entity(*entity).remove::<ToBeDespawned>();
            *entity
        },
        None => {
            println!("Root node doesn't exist, spawning");
            let mut root_path = path.clone();
            let root_name = path.file_name().unwrap();
            root_path.pop();
            root_position = Vec2::ZERO;
            let root_type = get_type_from_path(&root_path).unwrap();
            println!("Root Path: {}, Root Name: {}", root_path.display(), root_name.to_string_lossy());
            spawn_node(
                &mut node_event,
                &mut commands, 
                root_path, 
                root_name.into(),
                root_type,
                root_position,
                &mut pe_index,
            )
        }
    };
    // Remove ContextRoot marker component from previous root
    for node in root.iter(){
        commands.entity(node).remove::<ContextRoot>();
    }
    commands.entity(root_node).insert(ContextRoot);
    commands.entity(root_node).insert(PinnedToPosition);

    // Get position 


    // Don't despawn the parent directory of the root
    let root_parent_path = path.parent().unwrap();

    // Spawn parent if it doesn't exist
    // AND if we are not already at the root

    let parent_node = match pe_index.0.get(root_parent_path) {
        Some(entity) => {
            println!("Parent node already exists");
            commands.entity(*entity).remove::<ToBeDespawned>();
            Some(*entity)
        },
        None => {
            if root_parent_path.starts_with(&vault.get_root_path()){
                println!("Parent node doesn't exist, spawning");

                let parent_name = root_parent_path.file_name().unwrap();
                let parent_path = root_parent_path.parent().unwrap();

                // Check if the parent is a directory or a file

                let parent_type = get_type_from_path(&path).unwrap();

                println!("Parent Path: {}, Parent Name: {:?}", parent_path.display(), parent_name);
                Some(spawn_node(
                    &mut node_event,
                    &mut commands, 
                    parent_path.into(), 
                    parent_name.into(),
                    parent_type,
                    root_position,
                    &mut pe_index,
                ))
            } else {
                println!("Current context is vault root, not spawning parent");
                None
            }
        }
    };

    // Add an edge from the parent to the root, if the parent exists
    match parent_node {
        Some(entity) => {
            create_edge(
                &mut edge_event,
                &entity, 
                &root_node, 
                EdgeTypes::Parent,
                &mut commands,
                &edges,
            );
        },
        None => (),
    }

    println!("Path: {}", path.display());
    let entries = fs::read_dir(&path);

    // Get all file and folder names in 
    let _entries = match entries {
        Ok(entries) => {
            // Get all files
            let mut file_names: Vec<OsString> = entries
            // Ignore the vault folder!!!
            .filter(|entry| {
                let path = entry.as_ref().unwrap().path();
                println!("Is path {:?} the vault path: {}:", path, path == vault.get_vault_path());
                path != vault.get_vault_path()
            })
            // Carry on with everything else
            .filter_map(|entry| {
                let path = entry.ok()?.path().clone();
                let file_name: OsString = path.file_name()?.into();
                Some(file_name)
            })
            .collect();

            // file_names.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
            // Sort the OsStr vec
            file_names.sort_by(|a, b| a.cmp(&b));

            for file in file_names.iter() {
                println!("File: {}", file.to_string_lossy());
            }

            file_names.iter().for_each(|name| {

                // Check if the item already exists
                let full_path = path.join(name);
                let item_exists = pe_index.0.get(&full_path).is_some();
                if item_exists {
                        println!("Item already exists: {}", full_path.display());
                        // Remove despawn component
                        commands.entity(pe_index.0.get(&full_path).unwrap().clone()).remove::<ToBeDespawned>();
                        return
                }
        
                // Get the type of the item
                let item_type = get_type_from_path(&full_path);
                let item_type = match item_type {
                    Some(item_type) => item_type,
                    None => {
                        println!("Item path unwrap panic: {}", full_path.display());
                        return
                    }
                };
        
                // Spawn a node for each item
                let node = spawn_node(
                    &mut node_event,
                    &mut commands,
                    path.clone().into(),
                    name.into(),
                    item_type,
                    root_position,
                    &mut pe_index,
                );
        
                // Spawn an edge from the root node to each item
                create_edge(
                    &mut edge_event,
                    &root_node, 
                    &node, 
                    EdgeTypes::Parent,
                    &mut commands,
                    &edges,
                );
            });
        },
        Err(e) => {
            println!("Error: {}", e);
        }
    };
    

    // Print pe_index to see what the hell is going on
    for (path, entity) in pe_index.0.iter() {
        println!("Path: {}, Entity: {:?}", path.display(), entity);
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

// --------------------------------------------------------------------------------
// TESTS
// --------------------------------------------------------------------------------

#[cfg(test)]
fn file_node_has_directory_parent() {
    // Create a file node
    // Check that it has a directory parent
}   