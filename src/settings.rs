use bevy::prelude::{Plugin, App};

use self::{view::EdgeDefaults, theme::ThemePlugin};

mod view;
pub mod theme;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(ThemePlugin)
            .insert_resource(EdgeDefaults::default())
        ;
    }
}