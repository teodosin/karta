//

use std::path::PathBuf;

use bevy::ecs::{event::Event, entity::Entity};

#[derive(Event)]
pub struct RequestContextExpand {
    pub target_path: PathBuf,
    pub target_entity: Entity,
    pub as_visitors: bool,
}

#[derive(Event)]
pub struct RequestContextCollapse {
    pub target_path: PathBuf,
    pub target_entity: Entity,
}