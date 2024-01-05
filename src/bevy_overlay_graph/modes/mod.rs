// Declarations of mode modules 

use std::fmt;

use bevy::prelude::{States, App, Plugin, Resource};

use self::{
    state::StatePlugin, 
    r#move::MovePlugin, edges::EdgesPlugin
};

pub mod r#move;
pub mod edges;
pub mod state;
pub mod play;

#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default)]
pub enum KartaModeState {
    #[default]
    Move,
    Edges,
    State,
    Play,
}

impl fmt::Display for KartaModeState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            KartaModeState::Move => write!(f, "Move"),
            KartaModeState::Edges => write!(f, "Edges"),
            KartaModeState::State => write!(f, "Context"),
            KartaModeState::Play => write!(f, "Play"),
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
            .add_plugins(MovePlugin)
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
            mode: KartaModeState::Move,
        }
    }
}