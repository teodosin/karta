/// Main file for the Context plugin
/// The Context manages what is in the node graph. 
/// Responsible for keeping track of what is spawned and despawned.

use bevy::{prelude::*, utils::HashMap};
use bevy_mod_picking::prelude::PointerButton;
use std::fs;

use crate::{
    graph::{graph_cam, edges::create_edge}, vault::KartaVault, 
    events::nodes::NodeClickEvent, input::pointer::InputData
};

use super::nodes::*;

pub struct ContextPlugin;

impl Plugin for ContextPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(PathsToEntitiesIndex(HashMap::new()))
            .insert_resource(CurrentContext::new())

            .add_systems(Startup, initial_context)

            .add_systems(Update, update_context
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
        button: PointerButton::Primary,
    });
}

// Big monolith function
// --------------------------------------------------------------------------------
pub fn update_context(
    input_data: Res<InputData>,

    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,

    vault: Res<KartaVault>,
    context: Res<CurrentContext>,

    mut view_data: ResMut<graph_cam::ViewData>,
    mut pe_index: ResMut<PathsToEntitiesIndex>,

    mut nodes: Query<(Entity, &GraphDataNode, &Transform)>,
) {
    // Handle previous context
    //----------------------------------------
    let previous_root = pe_index.0.get(&context.get_current_context_path());
    match previous_root {
        Some(entity) => {
            //println!("Previous root: {:?}", entity);
            commands.entity(*entity).remove::<ContextRoot>();
        },
        None => (),
    }    


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
            let root_name = path.split("/").last().unwrap().to_string();
            let root_path = path.replace(&root_name, "");
            let root_path = &root_path[0..&root_path.len()-1].to_string();
            root_position = Vec2::ZERO;
            println!("Root Path: {}, Root Name: {}", root_path, root_name);
            spawn_node(
                &mut commands, 
                &root_path, 
                &root_name,
                root_position,
                &mut meshes, 
                &mut materials, 
                &mut view_data,
                &mut pe_index,
            )
        }
    };
    commands.entity(root_node).insert(ContextRoot);

    // Get position 


    // Don't despawn the parent of the root
    let root_parent_path = path
        .replace(&path.split("/")
        .last()
        .unwrap(), "");
    let root_parent_path = &root_parent_path[0..&root_parent_path.len()-1].to_string();

    // Spawn parent if it doesn't exist
    // AND if we are not already at the root
    let mut parent_node = Option::<Entity>::None;

    parent_node = match pe_index.0.get(root_parent_path) {
        Some(entity) => {
            println!("Parent node already exists");
            commands.entity(*entity).remove::<ToBeDespawned>();
            Some(*entity)
        },
        None => {
            if root_parent_path.contains(&vault.get_root_path()){
                println!("Parent node doesn't exist, spawning");
                let parent_name = root_parent_path.split("/").last().unwrap().to_string();
                let parent_path = root_parent_path.replace(&parent_name, "");
                let parent_path = &parent_path[0..&parent_path.len()-1].to_string();
                println!("Parent Path: {}, Parent Name: {}", parent_path, parent_name);
                Some(spawn_node(
                    &mut commands, 
                    &parent_path, 
                    &parent_name,
                    root_position,
                    &mut meshes, 
                    &mut materials, 
                    &mut view_data,
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
                &entity, 
                &root_node, 
                &mut commands,
                &mut view_data
            );
        },
        None => (),
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
            root_position,
            &mut meshes,
            &mut materials,
            &mut view_data,
            &mut pe_index,
        );

        // Spawn an edge from the root node to each item
        create_edge(
            &root_node, 
            &node, 
            &mut commands,
            &mut view_data
        );
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

