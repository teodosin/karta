// Declarations of mode modules 

use bevy::prelude::{States, App, Plugin};

pub mod arrange;
pub mod select;

#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default)]
pub enum KartaModeState {
    #[default]
    Arrange,
    Select,
}

pub struct ModePlugin;

impl Plugin for ModePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_state::<KartaModeState>()
        ;
    }
}