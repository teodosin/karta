// Declarations of tool modules 

use bevy::prelude::States;

pub mod arrange;
pub mod select;

#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default)]
pub enum KartaToolState {
    #[default]
    Arrange,
    Select,
}

