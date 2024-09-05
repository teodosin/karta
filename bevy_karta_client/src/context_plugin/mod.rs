//TODO: Would using Bevy's Hashmap be better for performance?
use std::{collections::HashMap, path::{Path, PathBuf}};

use bevy::prelude::*;
use fs_graph::prelude::*;

use crate::prelude::CurrentVault;

// -----------------------------------------------------------------
// Plugin
pub struct ContextPlugin;

impl Plugin for ContextPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(CurrentContext::empty())
            .insert_resource(PathsToEntitiesIndex::new())

            .add_systems(PreUpdate, on_context_change.run_if(resource_changed::<CurrentContext>))
        ;
    }
}

// -----------------------------------------------------------------
// Resources
#[derive(Resource)]
pub struct CurrentContext {
    undo_stack: Vec<KartaContext>,
    redo_stack: Vec<KartaContext>,
    setting_from_undo_redo: bool,

    context: Option<KartaContext>,
}

impl CurrentContext {
    pub fn empty() -> Self {
        CurrentContext {
            context: None,

            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            setting_from_undo_redo: false,
        }
    }

    pub fn set(&mut self, ctx_path: NodePath){
        self.context = Some(KartaContext {
            path: ctx_path,
        });
    }
}

/// Index that allows for quick lookup of node entities by their path.
/// Must be updated every time a node is spawned or despawned. 
/// Can be used to quickly check for whether a node is spawned or not. 
#[derive(Resource, Debug)]
pub struct PathsToEntitiesIndex(
    HashMap<NodePath, Entity>,
);

impl PathsToEntitiesIndex {
    pub fn new() -> Self {
        PathsToEntitiesIndex(HashMap::default())
    }

    pub fn get_entity_from_path(&self, path: &NodePath) -> Option<Entity> {
        self.0.get(path).cloned()
    }
}

pub struct KartaContext {
    path: NodePath, 
}

impl KartaContext {
    pub fn root() -> Self{
        KartaContext {
            path: NodePath::root(),
        }
    }

    pub fn user_root() -> Self {
        KartaContext {
            path: NodePath::user_root(),
        }
    }
}

// Events
// --------------------------------------------------------------------
// How many of these are necessary, when systems can use change detection?

#[derive(Event)]
pub struct NodeSpawnedEvent {}

#[derive(Event)]
pub struct NodeDespawnedEvent {}


#[derive(Event)]
pub struct NodeCreatedEvent {} 

#[derive(Event)]
pub struct NodeDeletedEvent {}


#[derive(Event)]
pub struct EdgeCreatedEvent {}

#[derive(Event)]
pub struct EdgeDeletedEvent {}


// --------------------------------------------------------------------
// Systems

fn on_context_change(
    context: Res<CurrentContext>,
    vault: Res<CurrentVault>,
){
    let graph = match &vault.graph {
        Some(graph) => graph,
        None => return, // TODO: Better error handling here
    };

    let ctx = match &context.context {
        Some(ctx) => ctx,
        None => return, // TODO: Better error handling here
    };
    
    let ctx_root_nodepath = ctx.path.clone();

    let node = graph.open_node(&ctx_root_nodepath);

    match node {
        Ok(node) => {
            println!("Node found: {:#?}", node);
        },
        Err(e) => {
            println!("Node not found: {}", e);
        }
    }


}