//

use std::path::PathBuf;

use bevy::ecs::event::Event;

#[derive(Event)]
pub struct RequestContextExpand {
    pub target: Option<PathBuf>,
}