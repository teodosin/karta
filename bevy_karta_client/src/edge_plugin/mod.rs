use bevy::prelude::*;
use karta_server::prelude::*;

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




// #[derive(Component, Deref)]
// pub struct DataEdgeType(pub EdgeType);


#[derive(Bundle)]
/// Bevy 0.15 TODO: convert to use required components
pub struct DataEdgeBundle {
    pub data_edge: DataEdge,
    pub attributes: Attributes,
}
