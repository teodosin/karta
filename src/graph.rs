pub mod graph_cam;
pub mod context;

pub mod nodes;
pub mod node_types;
pub mod edges;
pub mod attribute;

mod simulation;
mod quadtree;

use bevy::prelude::*;

pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(graph_cam::GraphCamPlugin)
            .add_plugins(context::ContextPlugin)
            .add_plugins(nodes::NodesPlugin)
            .add_plugins(edges::EdgesPlugin)

            .add_plugins(node_types::NodeTypesPlugin)

            .add_plugins(simulation::GraphSimPlugin)

            .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.12)))

            .add_systems(Startup, setup_grid)

        ;
    }
}

fn setup_grid(
    mut commands: Commands
){
    // commands.spawn(InfiniteGrid2DBundle::default());
}