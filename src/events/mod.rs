use bevy::prelude::{Plugin, App};

use self::context::*;

pub mod context;


pub struct KartaEventPlugin;

impl Plugin for KartaEventPlugin {
    fn build(&self, app: &mut App) {
        app

            // Context events
            .add_event::<RequestContextExpand>()
            .add_event::<RequestContextCollapse>()
        ;
    }
}