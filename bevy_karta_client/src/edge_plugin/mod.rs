use bevy::prelude::*;
use fs_graph::prelude::*;

use crate::prelude::Attributes;

pub struct DataEdgePlugin;

// impl Plugin for DataEdgePlugin {
//     fn build(&self, app: &mut App) {
//         app
//             .add_systems(Startup, spawn_data_edge);
//     }
// }

#[derive(Component)]
pub struct DataEdge {
    pub source: NodePath,
    pub target: NodePath,
    pub created_time: SysTime,
    pub modified_time: SysTime,
}


pub struct Relation {
    node: Entity,
    edge: Entity,
    is_source: bool,
}

impl Relation {
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
    edges: Vec<Relation>,
}

impl Relations {
    pub fn new() -> Self {
        Self {
            edges: Vec::new(),
        }
    }

    pub fn add(&mut self, edge: Relation){
        self.edges.push(edge);
    }
}

// #[derive(Component, Deref)]
// pub struct DataEdgeType(pub EdgeType);


#[derive(Bundle)]
/// Bevy 0.15 TODO: convert to use required components
pub struct DataEdgeBundle {
    pub data_edge: DataEdge,
    pub attributes: Attributes,
}
