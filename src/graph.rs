pub mod graph_cam;
pub mod context;
pub mod nodes;
pub mod edges;
pub mod grid;

use bevy::prelude::*;

use self::grid::InfiniteGrid2DBundle;

pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(graph_cam::GraphCamPlugin)
            .add_plugins(context::ContextPlugin)
            .add_plugins(nodes::NodesPlugin)
            .add_plugins(edges::EdgesPlugin)

            .add_systems(Startup, setup_grid)

        ;
    }
}

fn setup_grid(
    mut commands: Commands
){
    commands.spawn(InfiniteGrid2DBundle::default());
}