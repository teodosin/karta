// Declarations of tool modules 

use bevy::prelude::{States, App, Plugin};

pub mod arrange;
pub mod select;

#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default)]
pub enum KartaToolState {
    #[default]
    Arrange,
    Select,
}

pub struct ToolPlugin;

impl Plugin for ToolPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_state::<KartaToolState>()
        ;
    }
}