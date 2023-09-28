// Declarations of mode modules 

use std::fmt;

use bevy::prelude::{States, App, Plugin, Resource};

use self::{
    state::StatePlugin, 
    arrange::ArrangePlugin, edges::EdgesPlugin
};

pub mod arrange;
pub mod select;
pub mod edges;
pub mod state;

#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default)]
pub enum KartaModeState {
    #[default]
    Arrange,
    Select,
    Edges,
    State,
}

impl fmt::Display for KartaModeState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            KartaModeState::Arrange => write!(f, "Arrange"),
            KartaModeState::Select => write!(f, "Select"),
            KartaModeState::Edges => write!(f, "Edges"),
            KartaModeState::State => write!(f, "Context"),
        }
    }
}

pub struct ModePlugin;

impl Plugin for ModePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_state::<KartaModeState>()

            .insert_resource(ActiveMode::default())

            .add_plugins(StatePlugin)
            .add_plugins(ArrangePlugin)
            .add_plugins(EdgesPlugin)
        ;
    }
}

#[derive(Resource)]
pub struct ActiveMode {
    pub mode: KartaModeState,
}

impl Default for ActiveMode {
    fn default() -> Self {
        ActiveMode {
            mode: KartaModeState::Arrange,
        }
    }
}