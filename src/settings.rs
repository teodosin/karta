use bevy::prelude::{Plugin, App};

use self::view::EdgeDefaults;

mod view;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(EdgeDefaults::default())
        ;
    }
}