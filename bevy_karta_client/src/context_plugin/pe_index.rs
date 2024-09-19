use crate::prelude::{DataEdge, DataNode, Relation, Relations, ViewNode};
use bevy::{prelude::*, utils::HashMap};
use fs_graph::prelude::NodePath;

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
    pub HashMap<NodePath, NodeEntityPair>,
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

pub fn update_pe_index_on_datanode_spawn(
    mut nodes: Query<(Entity, &DataNode), Added<DataNode>>,
    mut pe_index: ResMut<PathsToEntitiesIndex>,
){
    for (entity, node) in nodes.iter_mut() {
        // println!("Adding to pe_index: {:?} | {:?}", node.path.clone(), entity);
        
        pe_index.add_data(node.path.clone(), entity);
    }
}

pub fn update_pe_index_on_viewnode_spawn(
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

pub fn update_relations_on_edge_spawn(
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