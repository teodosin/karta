use bevy::prelude::*;
use fs_graph::prelude::*;

use bevy_overlay_graph::prelude::*;

pub struct DataNodePlugin;

impl Plugin for DataNodePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, spawn_data_node);
    }
}

#[derive(Component)]
pub struct DataNode {
    pub path: NodePath,
    pub created_time: SysTime,
    pub modified_time: SysTime,
}

#[derive(Component, Deref)]
pub struct DataNodeType(pub NodeType);

#[derive(Component, Deref)]
pub struct Attributes(pub Vec<Attribute>);

#[derive(Bundle)]
/// Bevy 0.15 TODO: convert to use required components
pub struct DataNodeBundle {
    pub name: Name,
    pub data_node: DataNode,
    pub data_node_type: DataNodeType,
    pub attributes: Attributes,
    pub ui_marker: GraphEntity,
}

fn spawn_data_node(){}