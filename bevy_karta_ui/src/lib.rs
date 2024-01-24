// Root file of the bevy_overlay_graph crate.

use bevy::{prelude::*, asset::embedded_asset};
use bevy_mod_picking::{DefaultPickingPlugins, debug::DebugPickingPlugin};

pub mod core;
pub mod assets;
pub mod settings;
pub mod events;
pub mod input;
pub mod ui;

// Common imports
pub mod prelude {
    pub use crate::core::*;
    pub use crate::assets::*;
    pub use crate::settings::*;
    pub use crate::events::*;
    pub use crate::input::*;
    pub use crate::ui::*;
}
pub struct OverlayGraphPlugin;

impl Plugin for OverlayGraphPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(DefaultPickingPlugins
                .build()
                .disable::<DebugPickingPlugin>()
            )        
            // .add_plugins(EmbeddedAssetPlugin{
            //     mode: bevy_embedded_assets::PluginMode::AutoLoad,
            // })

            .add_plugins(core::CorePlugin)

            .add_plugins(assets::AssetsPlugin)
        
            .add_plugins(settings::SettingsPlugin)

            .add_plugins(input::InputPlugin)

            .add_plugins(ui::KartaUiPlugin)

            .add_plugins(events::EventPlugin)
        ;
        // embedded_asset!(app, "../assets/grid_material.wgsl");

    }
}