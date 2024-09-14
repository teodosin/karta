//TODO: Would using Bevy's Hashmap be better for performance?
use std::{collections::HashMap, path::{Path, PathBuf}};

use bevy::prelude::*;
use fs_graph::prelude::*;

use crate::{node_plugin, prelude::{CurrentVault, DataEdge, DataEdgeBundle, DataNode, DataNodeBundle, Relation, Relations, ViewNode}};

// -----------------------------------------------------------------
// Plugin
pub struct ContextPlugin;

impl Plugin for ContextPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(CurrentContext::empty())
            .insert_resource(PathsToEntitiesIndex::new())

            .add_systems(PreUpdate, on_context_change.run_if(resource_changed::<CurrentContext>))

            .add_systems(Update, (
                update_pe_index_on_datanode_spawn,
                update_pe_index_on_viewnode_spawn,
                update_relations_on_edge_spawn,
            ).chain())
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

/// Struct for keeping together a DataNode and its corresponding
/// ViewNode. Mostly used in the PathsToEntitiesIndex.
#[derive(Debug)]
pub struct NodeEntityPair {
    data: Option<Entity>,
    view: Option<Entity>,
}

impl NodeEntityPair {
    pub fn empty() -> Self {
        NodeEntityPair {
            data: None,
            view: None,
        }
    }

    pub fn new(data: Entity, view: Entity) -> Self {
        NodeEntityPair {
            data: Some(data),
            view: Some(view),
        }
    }

    pub fn new_data(data: Entity) -> Self {
        NodeEntityPair {
            data: Some(data),
            view: None,
        }
    }

    pub fn new_view(view: Entity) -> Self {
        NodeEntityPair {
            data: None,
            view: Some(view),
        }
    }

    pub fn update_data(&mut self, data: Entity) {
        self.data = Some(data);
    }

    pub fn update_view(&mut self, view: Entity) {
        self.view = Some(view);
    }
}

/// Index that allows for quick lookup of node entities by their path.
/// Must be updated every time a node is spawned or despawned. 
/// Can be used to quickly check for whether a node is spawned or not. 
#[derive(Resource, Debug)]
pub struct PathsToEntitiesIndex(
    HashMap<NodePath, NodeEntityPair>,
);

impl PathsToEntitiesIndex {
    pub fn new() -> Self {
        PathsToEntitiesIndex(HashMap::default())
    }

    pub fn add_pair(&mut self, path: NodePath, entities: NodeEntityPair){
        self.0.insert(path, entities);
    }

    pub fn add_view(&mut self, path: NodePath, view: Entity){
        let pair = self.0.get_mut(&path);
        match pair {
            Some(pair) => pair.view = Some(view),
            None => {
                let pair = NodeEntityPair::new_view(view);
                self.0.insert(path, pair);
            },
        }
    }

    pub fn add_data(&mut self, path: NodePath, data: Entity){
        let pair = self.0.get_mut(&path);
        match pair {
            Some(pair) => pair.data = Some(data),
            None => {
                let pair = NodeEntityPair::new_data(data);
                self.0.insert(path, pair);
            },
        }
    }

    pub fn remove(&mut self, path: &NodePath) -> Option<NodeEntityPair> {
        self.0.remove(path)
    }

    pub fn get_pair(&self, path: &NodePath) -> Option<&NodeEntityPair> {
        self.0.get(path)
    }

    pub fn get_view(&self, path: &NodePath) -> Option<Entity> {
        let pair = self.0.get(path);
        match pair {
            Some(pair) => pair.view.clone(),
            None => None,
        }
    }

    pub fn get_data(&self, path: &NodePath) -> Option<Entity> {
        let pair = self.0.get(path);
        match pair {
            Some(pair) => pair.data.clone(),
            None => None,
        }
    }
}

fn update_pe_index_on_datanode_spawn(
    mut nodes: Query<(Entity, &DataNode), Added<DataNode>>,
    mut pe_index: ResMut<PathsToEntitiesIndex>,
){
    for (entity, node) in nodes.iter_mut() {
        // println!("Adding to pe_index: {:?} | {:?}", node.path.clone(), entity);
        
        pe_index.add_data(node.path.clone(), entity);
    }
}

fn update_pe_index_on_viewnode_spawn(
    mut nodes: Query<(Entity, &ViewNode), Added<ViewNode>>,
    mut pe_index: ResMut<PathsToEntitiesIndex>,
){
    for (entity, node) in nodes.iter_mut() {
        println!("Adding to pe_index: {:?} | {:?}", node.path.clone(), entity);
        let path = node.path.clone();
        let path = match path {
            Some(path) => path,
            None => continue,
        };
        pe_index.add_view(path, entity);
    }
}

fn update_relations_on_edge_spawn(
    edges: Query<(Entity, &DataEdge), Added<DataEdge>>,
    mut nodes: Query<(Entity, &mut Relations)>,
    pe_index: Res<PathsToEntitiesIndex>,
){
    for (edge_e, edge) in edges.iter() {
        let source_path = edge.source.clone();
        let target_path = edge.target.clone();

        let source_e = match pe_index.get_view(&source_path){
            Some(e) => e,
            None => {
                warn!("Could not find source node for edge {edge_e}");
                continue;
            },
        };
        let target_e = match pe_index.get_view(&target_path) {
            Some(e) => e,
            None => {
                warn!("Could not find target node for edge {edge_e}");
                continue;
            },
        };

        match nodes.get_mut(source_e) {
            Ok((_, mut relations)) => {
                relations.add(Relation::new(target_e, edge_e, true));
            },
            Err(_) => {
                warn!("Could not find source node for edge {edge_e}");
            },
        }

        match nodes.get_mut(target_e) {
            Ok((_, mut relations)) => {
                relations.add(Relation::new(source_e, edge_e, false));
            },
            Err(_) => {
                warn!("Could not find target node for edge {edge_e}");
            },
        }
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

// --------------------------------------------------------------------
// Systems

fn on_context_change(
    mut commands: Commands,
    context: Res<CurrentContext>,
    mut vault: ResMut<CurrentVault>,
){
    let graph = match &mut vault.graph {
        Some(graph) => graph,
        None => return, // TODO: Better error handling here
    };

    let ctx = match &context.context {
        Some(ctx) => ctx,
        None => return, // TODO: Better error handling here
    };
    
    let ctx_root_nodepath = ctx.path.clone();

    
    let node = graph.open_node(&ctx_root_nodepath);



    let node: fs_graph::prelude::Node = match node {
        Ok(node) => {
            println!("Node found: {:#?}", node);
            node
        },
        Err(e) => {
            println!("Node not found: {}", e);
            return;
        }
    };
    
    let nodepath = node.path().clone();
    let name = node.name().to_string();

    commands.spawn(DataNodeBundle{
        name: Name::new(name),
        data_node: DataNode {
            path: nodepath,
            created_time: node.created_time().clone(),
            modified_time: node.modified_time().clone(),
        },
        data_node_type: node_plugin::DataNodeType(node.ntype_name()),
        attributes: node_plugin::Attributes(node.attributes().clone()),
    });

    let connections: Vec<(fs_graph::prelude::Node, Edge)> = graph.open_node_connections(&ctx_root_nodepath);

    for node in connections.iter() {
        let (node, edge) = node;
        let node_path = node.path().clone();
        let name = node.name().to_string();

        commands.spawn(DataNodeBundle{
            name: Name::new(name),
            data_node: DataNode {
                path: node_path,
                created_time: node.created_time().clone(),
                modified_time: node.modified_time().clone(),
            },
            data_node_type: node_plugin::DataNodeType(node.ntype_name()),
            attributes: node_plugin::Attributes(node.attributes().clone()),
        });

        commands.spawn(DataEdgeBundle {
            data_edge: DataEdge {
                source: edge.source().clone(),
                target: edge.target().clone(),
                created_time: edge.created_time().clone(),
                modified_time: edge.modified_time().clone(),
            },
            attributes: node_plugin::Attributes(edge.attributes().clone()),
        });
    }
}