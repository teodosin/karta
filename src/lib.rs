//lib

use bevy::{prelude::*, log::LogPlugin, window::WindowResolution};
use bevy_fs_graph::prelude::*;
use bevy_overlay_graph::prelude::*;




pub fn karta_app() {
    let mut app = App::new();

    app
        // PLUGIN BLOCK
        .add_plugins(DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Karta".to_string(),
                    resolution: WindowResolution::new(1920., 1080.),
                    ..default()
                }),
                ..default()
            })
            .build()
            .disable::<LogPlugin>()
        )

        // .add_plugins(EmbeddedAssetPlugin{
        //     mode: bevy_embedded_assets::PluginMode::AutoLoad,
        // })

        
        .add_plugins(KartaCorePlugin)
        .add_plugins(OverlayGraphPlugin)
    ;

    app.run();
}








