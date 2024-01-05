// Root file of the bevy_overlay_graph crate.
// It begins its life as mod.rs before becoming lib.rs.

use bevy::prelude::*;

pub mod settings;
pub mod input;
pub mod ui;
pub mod modes;
pub mod events;

pub struct OverlayGraphPlugin;

impl Plugin for OverlayGraphPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(settings::SettingsPlugin)

            .add_plugins(input::InputPlugin)

            .add_plugins(ui::KartaUiPlugin)

            .add_plugins(modes::ModePlugin)

            .add_plugins(events::EventPlugin)

        ;
    }
}

