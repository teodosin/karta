use bevy::app::{App, Plugin};

use crate::vault_plugin::VaultPlugin;


pub struct KartaCorePlugin;

impl Plugin for KartaCorePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(VaultPlugin)
        ;
    }
}