use bevy::{app::{App, Plugin}, prelude::Event};
use fs_graph::prelude::NodePath;

pub struct ContextEventsPlugin;

impl Plugin for ContextEventsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<ChangeContextEvent>();
    }
}


#[derive(Event)]
pub struct ChangeContextEvent {
    pub new_ctx: NodePath,
}