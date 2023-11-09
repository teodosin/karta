//

use bevy::prelude::{Event, Entity};

use crate::graph::edges::EdgeTypes;

#[derive(Event)]
pub struct EdgeSpawnedEvent {
    pub entity: Entity,
    pub connected_to_focal: bool,
    pub edge_type: EdgeTypes,
}