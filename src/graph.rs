pub mod graph_cam;
pub mod context;
pub mod nodes;
pub mod edges;

use bevy::prelude::*;

pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(graph_cam::GraphCamPlugin)
            .add_plugins(context::ContextPlugin)
            .add_plugins(nodes::NodesPlugin)
            .add_plugins(edges::EdgesPlugin)


        ;
    }
}