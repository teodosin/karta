use bevy::prelude::*;
use fs_graph::prelude::*;

use bevy_overlay_graph::prelude::*;

pub struct DataNodePlugin;

impl Plugin for DataNodePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(spawn_data_node);
    }
}

#[derive(Component)]
pub struct DataNode {
    pub path: NodePath,
}

fn spawn_data_node(){}