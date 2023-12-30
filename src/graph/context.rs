/// Main file for the Context plugin
/// The Context manages what is in the node graph. 
/// Responsible for keeping track of what is spawned and despawned.

use bevy::{prelude::*, utils::{HashMap, HashSet}};
use bevy_mod_picking::prelude::PointerButton;
use std::{fs, path::PathBuf, ffi::OsString};

use crate::{
    graph::{graph_cam, edges::{create_edge, EdgeTypes}, node_types::{get_type_from_file_path, NodeTypes, get_type_from_context_path}}, vault::{CurrentVault, context_asset::{open_context_file_from_node_path, ContextAsset, node_path_to_context_path, open_context_file}}, 
    events::{nodes::{NodeClickEvent, NodeSpawnedEvent}, edges::EdgeSpawnedEvent}, ui::nodes::TargetPosition,
};

use super::{nodes::*, edges::{GraphDataEdge, EdgeType}};

pub struct ContextPlugin;

impl Plugin for ContextPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(PathsToEntitiesIndex(HashMap::new()))
            .insert_resource(CurrentContext::new())

            // .add_systems(Startup, initial_context)

            .add_systems(Last, update_context
                .run_if(resource_changed::<CurrentContext>())
            )
        ;
    }
}

/// Index that allows for quick lookup of node entities by their path.
/// Must be updated every time a node is spawned or despawned. 
/// Can be used to quickly check for whether a node is spawned or not. 
#[derive(Resource, Debug)]
pub struct PathsToEntitiesIndex(
    pub HashMap<PathBuf, Entity>,
);

/// Enum for distinguishing between whether a node refers to a physical file/folder
/// or a virtual node defined inside a context. 
#[derive(Debug, PartialEq, Clone)]
pub enum ContextTypes {
    Unknown,
    Physical, 
    Virtual,
}

/// The resource that stores the current context path.
/// A context can be any file or node defined inside some context. 
/// Any node that is created which is not a file in itself, must still
/// be stored somewhere. That place is the closest parent folder. 
#[derive(Resource, Debug)]
pub struct CurrentContext{
    undo_stack: Vec<PathBuf>,
    redo_stack: Vec<PathBuf>,
    pub context: Option<KartaContext>,
}

impl CurrentContext {
    fn new() -> Self {
        CurrentContext {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            context: None,
        }
    }

    /// Main function for setting the current context. Only use this, never mutate the resource directly.
    pub fn set_current_context(&mut self, vault_path: PathBuf, path: PathBuf) {

        println!("Trying to set context: {}", path.display());

        // If path is outside of vault, return
        if !path.starts_with(&vault_path.parent().unwrap()) {
            println!("Path {} is outside of vault: {}", path.display(), vault_path.parent().unwrap().display());
            return
        }

        let mut path = path;
        let ctype: ContextTypes;

        // Check if the path already includes the vault path in case the path is already defined 
        // as absolute. 
        if !path.starts_with(&vault_path) {
            path = vault_path.join(path);
        }

        
        // Determine whether the context is a physical file or a virtual node
        if path.exists() {
            ctype = ContextTypes::Physical;
            println!("Path before canonicalize: {}", path.display());
            path = path.canonicalize().unwrap();
            println!("Path after canonicalize: {}", path.display());
        } else {
            ctype = ContextTypes::Virtual;
        }


        // Check if the path is a file or a directory
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
                        println!("Path is a directory, deeper: {}", dir_path.display());
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
        self.undo_stack.push(path.clone());
        self.context = Some(KartaContext::new(path).set_type(ctype));

    }

    pub fn undo_context(&mut self, vault_path: PathBuf) {
        if self.undo_stack.len() > 1 {
            let path = self.undo_stack.pop().unwrap();
            self.redo_stack.push(path.clone());
            let new_path = self.undo_stack.last().unwrap().clone();
            self.set_current_context(vault_path, new_path);
        }
    }

    pub fn redo_context(&mut self, vault_path: PathBuf) {
        if !self.redo_stack.is_empty() {
            let path = self.redo_stack.pop().unwrap();
            self.set_current_context(vault_path, path);
        }
    }
}

/// Main struct that defines a context.
#[derive(Debug, PartialEq, Clone)]
pub struct KartaContext {
    /// Path to the current context, including the file or dir name.
    path: PathBuf,
    ctype: ContextTypes,
}

impl KartaContext {
    pub fn new(path: PathBuf) -> Self {
        KartaContext {
            path,
            ctype: ContextTypes::Unknown,
        }
    }

    pub fn set_type(&mut self, ctype: ContextTypes) -> Self {
        KartaContext {
            path: self.path.clone(),
            ctype,
        }
    }

    pub fn get_type(&self) -> ContextTypes {
        self.ctype.clone()
    }

    pub fn get_path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn get_parent_path(&self) -> PathBuf {
        let mut parent_path = self.path.clone();
        parent_path.pop();
        parent_path
    }
}


/// A marker component for selected graph entities
/// Might be redundant, but I don't know bevy_mod_picking's implementation well enough yet
#[derive(Component, Clone)]
pub struct Selected;

/// Marker component for which nodes are to be despawned. 
/// Nodes are despawned in a separate system from the one that updates the context.
#[derive(Component)]
pub struct ToBeDespawned;


// --------------------------------------------------------------------------------
/// Big monolith function for updating the context. 
pub fn update_context(
    mut node_event: EventWriter<NodeSpawnedEvent>,
    mut edge_event: EventWriter<EdgeSpawnedEvent>,
    
    mut commands: Commands,

    vault: Res<CurrentVault>,
    context: Res<CurrentContext>,

    mut pe_index: ResMut<PathsToEntitiesIndex>,

    mut nodes: Query<(Entity, &GraphDataNode, &Transform)>,
    edges: Query<(&GraphDataEdge, &EdgeType)>,
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

    // Handle the path to the desired context. Updating the context resource
    // is what triggers this system to run, so the newest context is in there 
    // already.
    let (new_cxt_path, new_ctype) = match context.context {
        Some(ref cxt) => (cxt.get_path(), cxt.get_type()),
        None => {
            println!("No context set");
            return
        }
    };

    println!("Updating context: {}", new_cxt_path.display());

    // Get context file result
    let context_path = node_path_to_context_path(&vault.get_vault_path(), &new_cxt_path);
    let context_file = open_context_file(&context_path);

    // Uncomment this line to force an error
    // let context_file: Result<ContextAsset, String> = Err("".into());


    // Iterate through existing nodes and mark them for deletion. The nodes that shouldn't be 
    // deleted will have their ToBeDespawned component removed later in the function. 
    for (entity, _node, _pos) in nodes.iter_mut() {
        commands.entity(entity).insert(ToBeDespawned);
    }

    // Nodes will be spawned immediately, but edges will be spawned later. This is because 
    // edges can only be spawned between nodes that already exist, since the create_edge function 
    // takes Entities as arguments. 
    let mut nodes_spawned: HashMap<PathBuf, Entity> = HashMap::new();

    let mut edges_to_spawn: HashSet<(String, String)> = HashSet::new();    


    // All other nodes' positions will be relative to the root node's position, so we need to track it. 
    let root_position: Vec2;

    // ----------------- Handle context root -----------------
    let root_node: Entity = match pe_index.0.get(&new_cxt_path) {
        Some(entity) => {
    
            debug!("Root node already exists");
            root_position = nodes.get(*entity).unwrap().2.translation.truncate();
            commands.entity(*entity).remove::<ToBeDespawned>();

            match &context_file {
                Ok(context_file) => {
                    debug!("Context file exists for already existing node {}", new_cxt_path.display());

                    let root_in_file = context_file.nself.clone();
                    let root_path: PathBuf = root_in_file.path.into();
                    let root_path_str = root_path.to_string_lossy().to_string();

                    // Take into account both the nodes in the new context file as well as the edges.
                    // Nodes in a context file only get updated when the context gets saved with that
                    // node as root. If the node is not the context root, only its edges will get updated. 

                    for node in context_file.nodes.iter(){
                        let path = PathBuf::from(&node.path);
                        if pe_index.0.contains_key(&path){
                            let entity = pe_index.0.get(&path).unwrap();
                            commands.entity(*entity).remove::<ToBeDespawned>();
                            match node.relative_position {
                                Some(pos) => {
                                    println!("Inserting target position for {}", node.path);
                                    commands.entity(*entity).insert(TargetPosition {
                                        position: root_position + pos,
                                    });
                                },
                                None => {}
                            }
                        }
                    }
                    
                    for edge in context_file.edges.iter() {
                        edges_to_spawn.insert((edge.source.clone(), edge.target.clone()));
                        
                        // If this node is the source or target to a node that is already spawned,
                        // remove its ToBeDespawned component
                        println!("Pe index has {} amount of entries", pe_index.0.len());
                        if root_path_str == edge.source && pe_index.0.contains_key(&PathBuf::from(&edge.target)){
                            println!("Removing ToBeDespawned component from {}", edge.target);
                            commands.entity(*pe_index.0.get(&PathBuf::from(&edge.target)).unwrap()).remove::<ToBeDespawned>();
                        }
                        if root_path_str == edge.target && pe_index.0.contains_key(&PathBuf::from(&edge.source)){
                            println!("Removing ToBeDespawned component from {}", edge.source);
                            commands.entity(*pe_index.0.get(&PathBuf::from(&edge.source)).unwrap()).remove::<ToBeDespawned>();
                        }
                    }

                }
                Err(_e) => {
                }
                    
            }

            *entity

        },

        None => {
            match &context_file {
                Ok(context_file) => {
                    debug!("Context file exists for {}", new_cxt_path.display());

                    root_position = Vec2::ZERO;
                    let root_in_file = context_file.nself.clone();
                    let root_path: PathBuf = root_in_file.path.into();
                    let root_path_str = root_path.to_string_lossy().to_string();

                    println!("Amount of edges in new context: {}", context_file.edges.len());

                    for edge in context_file.edges.iter() {
                        edges_to_spawn.insert((edge.source.clone(), edge.target.clone()));
                        
                        // If this node is the source or target to a node that is already spawned,
                        // remove its ToBeDespawned component
                        println!("Pe index has {} amount of entries", pe_index.0.len());
                        if root_path_str == edge.source && pe_index.0.contains_key(&PathBuf::from(&edge.target)){
                            println!("Removing ToBeDespawned component from {}", edge.target);
                            commands.entity(*pe_index.0.get(&PathBuf::from(&edge.target)).unwrap()).remove::<ToBeDespawned>();
                        }
                        if root_path_str == edge.target && pe_index.0.contains_key(&PathBuf::from(&edge.source)){
                            println!("Removing ToBeDespawned component from {}", edge.source);
                            commands.entity(*pe_index.0.get(&PathBuf::from(&edge.source)).unwrap()).remove::<ToBeDespawned>();
                        }
                    }

                    spawn_node(
                        &mut node_event,
                        &mut commands, 
                        root_path.clone(), 
                        root_in_file.ntype,
                        root_position,
                        None,
                        root_in_file.pin_to_position,
                        &mut pe_index,
                    )
                },

                Err(_e) => {

                    debug!("Context file doesn't exist for {}", new_cxt_path.display());
                    let root_path = new_cxt_path.clone();
                    root_position = Vec2::ZERO;

                    let root_type = get_type_from_file_path(&root_path).unwrap();

                    let root_type = match new_ctype {
                        ContextTypes::Physical => root_type,
                        ContextTypes::Virtual => get_type_from_context_path(&context_path).unwrap(),
                        _ => NodeTypes::Base,
                    };

                    spawn_node(
                        &mut node_event,
                        &mut commands, 
                        root_path, 
                        root_type,
                        root_position,
                        None,
                        true,
                        &mut pe_index,
                    )
                }
            }
        }
    };

    nodes_spawned.insert(new_cxt_path.clone(), root_node);

    // Remove ContextRoot marker component from previous root
    for node in root.iter(){
        commands.entity(node).remove::<ContextRoot>();
    }
    commands.entity(root_node).insert(ContextRoot);


    // ----------------- Handle other nodes -----------------

    // Find the physical nodes that are parent or children of the current context. 
    // This vec will be used to compare against the context files, to keep the context files
    // in sync with the physical files. 
    let mut adjacent_physical_nodes: Vec<PathBuf> = Vec::new();

    // Spawn the remaining nodes. Implemented as a closure that captures the necessary variables
    // Only used in one place because of borrow issues in others. 
    let mut spawn_remaining_physicals = | physical_nodes: Vec<PathBuf> | {
        for node in physical_nodes.iter() {
            let node_path = node.clone();

            if nodes_spawned.contains_key(&node_path) {
                continue
            }

            let node_type = get_type_from_file_path(&node_path).unwrap();
            let node_pin_to_position = false;
            let spawned_node = spawn_node(
                &mut node_event,
                &mut commands, 
                node_path.clone(), 
                node_type,
                root_position,
                None,
                node_pin_to_position,
                &mut pe_index,
            );

            let edge_to_root = (node_path.clone().to_string_lossy().to_string(), new_cxt_path.clone().to_string_lossy().to_string());
            edges_to_spawn.insert(edge_to_root);

            // Add the node to the list of nodes that have been spawned
            nodes_spawned.insert(node_path, spawned_node);
        }
    };

    match new_ctype {
        // If the context is a physical file 
        ContextTypes::Physical => {
            if new_cxt_path.is_dir(){
                let parent: PathBuf = new_cxt_path.parent().unwrap().into();
                let parent_in_vault: bool = parent.starts_with(&vault.get_root_path());
                if parent.exists() && parent_in_vault {
                    adjacent_physical_nodes.push(parent.into());
                }

                let entries = fs::read_dir(&new_cxt_path);
                match entries {
                    Ok(entries) => {
                        entries
                            // Ignore the vault folder
                            .filter(|entry| {
                                let path = entry.as_ref().unwrap().path();
                                path != vault.get_vault_path()
                            })
                            .for_each(|entry| {
                                let path = entry.expect("Entries should be valid").path();
                                if path.exists(){
                                    adjacent_physical_nodes.push(path);
                                }
                            });
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            } else {
                let parent: PathBuf = new_cxt_path.parent().unwrap().into();
                let parent_in_vault: bool = parent.starts_with(&vault.get_root_path());
                if parent.exists() && parent_in_vault {
                    adjacent_physical_nodes.push(parent.into());
                }
            }
        }
        // Virtual nodes should be treated as if they exist in the file system, under closest parent folder.
        // TODO implement this behavior
        ContextTypes::Virtual => {

        }
        _ => {}
            
    }

    match context_file {
        Ok(context_file) => {
            // Spawn all nodes in the context file
            
            for node in context_file.nodes.iter() {
                
                println!("Node path in next context: {}", node.path);
                // Find the nodes that are already spawned and remove their ToBeDespawned component
                let node_path: PathBuf = node.path.clone().into();
                println!("Exists in pe_index: {}", pe_index.0.get(&node_path).is_some());
                if pe_index.0.get(&node_path).is_some() {
                    println!("Removing ToBeDespawned component from {}", node_path.display());
                    commands.entity(pe_index.0.get(&node_path).unwrap().clone()).remove::<ToBeDespawned>();
                    continue
                }

                let node_context_file = open_context_file_from_node_path(
                    &vault.get_vault_path(),
                    &node.path.clone().into()
                );

                match node_context_file {
                    Ok(node_context_file) => {
                        println!("Node context file exists");
                        let node_path: PathBuf = node.path.clone().into();
                        let node_type = node_context_file.nself.ntype;
                        let node_position = node.relative_position;
                        let node_pin_to_position = node.pin_to_position;
                        let spawned_node = spawn_node(
                            &mut node_event,
                            &mut commands, 
                            node_path.clone(), 
                            node_type,
                            root_position,
                            node_position,
                            node_pin_to_position,
                            &mut pe_index,
                        );

                        // Add the node to the list of nodes to spawn
                        nodes_spawned.insert(node_path, spawned_node);

                        // Add edges to the list of edges to spawn
                        for edge in node_context_file.edges.iter() {
                            edges_to_spawn.insert((edge.source.clone(), edge.target.clone()));
                        }

                    },

                    Err(_e) => {
                        println!("Node context file doesn't exist");
                        continue
                    }
                }
            }

            // Tried to use the spawn_remaining_physicals closure here, but there were borrow issues
            for node in adjacent_physical_nodes.iter() {
                let node_path = node.clone();

                if nodes_spawned.contains_key(&node_path) {
                    continue
                }

                let node_type = get_type_from_file_path(&node_path).unwrap();
                let node_pin_to_position = false;
                let spawned_node = spawn_node(
                    &mut node_event,
                    &mut commands, 
                    node_path.clone(), 
                    node_type,
                    root_position,
                    None,
                    node_pin_to_position,
                    &mut pe_index,
                );
                
    
                let edge_to_root = (node_path.clone().to_string_lossy().to_string(), new_cxt_path.clone().to_string_lossy().to_string());
                edges_to_spawn.insert(edge_to_root);
    
    
                // Add the node to the list of nodes that have been spawned
                nodes_spawned.insert(node_path, spawned_node);
            }

        },

        Err(_e) => {
            spawn_remaining_physicals(adjacent_physical_nodes);
        }
    }
    // Filter duplicate edges



    for edge in edges_to_spawn.iter() {
        let source_path = PathBuf::from(edge.0.clone());
        let target_path = PathBuf::from(edge.1.clone());

        let etype: EdgeTypes;

        let source_parent = source_path.parent().unwrap();
        let target_parent = target_path.parent().unwrap();

        let one_is_parent_of_other = source_parent == target_path || target_parent == source_path;

        // TODO: Should the source of an edge be the parent or the child in a parent connection?
        if one_is_parent_of_other {
            println!("Created edge of type PARENT");
            etype = EdgeTypes::Parent;
        } else {
            etype = EdgeTypes::Base;
        }

        create_edge(
            &mut edge_event,
            &source_path, 
            &target_path, 
            etype,
            &mut commands,
            &edges,
        );
    }
    
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
mod context_tests {
    use crate::tests::vault_utils::vault_utils;

    #[test]
    fn file_node_has_directory_parent() {
        let vault = vault_utils::get_test_vault("file_node_has_directory_parent");

        vault_utils::cleanup_test_vault(vault);
    }   

    // TEST: duplicate edges from context files are ignored when spawning edges

    // TEST: Set current context to context file with two nodes in context. 
    // Expected 3 GraphDataNode entities, 2 GraphEdge entities


}
