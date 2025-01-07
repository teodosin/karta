use bevy::prelude::*;
use karta_server::prelude::*;

// pub struct DataNodePlugin;

// impl Plugin for DataNodePlugin {
//     fn build(&self, app: &mut App) {
//         app
//             .add_systems(Startup, spawn_data_node);
//     }
// }

#[derive(Component)]
pub struct ToBeDespawned;

#[derive(Component)]
pub struct DataNode {
    pub path: NodePath,
    pub created_time: SysTime,
    pub modified_time: SysTime,
    pub persistent: bool, // Should persistency be stored on the node itself or in a separate table?
}

#[derive(Component, Deref)]
pub struct DataNodeType(pub NodeTypeId);

#[derive(Component, Deref)]
pub struct Attributes(pub Vec<Attribute>);

impl Attributes {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn get(&self, name: &str) -> Option<&Attribute> {
        self.0.iter().find(|attr| attr.name == name)
    }
}

#[derive(Bundle)]
/// Bevy 0.15 TODO: convert to use required components
pub struct DataNodeBundle {
    pub name: Name,
    pub data_node: DataNode,
    pub data_node_type: DataNodeType,
    pub attributes: Attributes,
}

#[derive(Component)]
pub struct ViewNode {
    pub path: Option<NodePath>,
    pub data: Option<Entity>,
}

pub struct NodeRelation {
    node: Entity,
    edge: Entity,
    is_source: bool,
}

impl NodeRelation {
    pub fn new(node: Entity, edge: Entity, is_source: bool) -> Self {
        Self {
            node,
            edge,
            is_source,
        }
    }
}

#[derive(Component)]
pub struct Relations {
    edges: Vec<NodeRelation>,
}

impl Relations {
    pub fn new() -> Self {
        Self {
            edges: Vec::new(),
        }
    }

    pub fn add(&mut self, edge: NodeRelation){
        self.edges.push(edge);
    }
}