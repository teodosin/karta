pub mod context;

pub mod nodes;
pub mod node_types;
pub mod edges;
pub mod attribute;

use bevy::prelude::*;

pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(context::ContextPlugin)
            .add_plugins(nodes::NodesPlugin)
            .add_plugins(edges::EdgesPlugin)
            
            .add_plugins(node_types::NodeTypesPlugin)

            .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.12)))

        ;
    }
}
