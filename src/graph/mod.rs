pub mod context;

pub mod nodes;
pub mod node_types;
pub mod edges;
pub mod attribute;
mod create_node_menu;

use bevy::prelude::*;

use self::create_node_menu::CreateNodeMenuPlugin;

pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(context::ContextPlugin)
            .add_plugins(nodes::NodesPlugin)
            .add_plugins(edges::EdgesPlugin)
            .add_plugins(CreateNodeMenuPlugin)
            
            .add_plugins(node_types::NodeTypesPlugin)

            .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.12)))

        ;
    }
}
