//lib

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::prelude::*;


mod ui;
mod vault;
mod scene;
mod graph;
mod modes;

pub fn karta_app() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(DefaultPickingPlugins
            .build()
            .disable::<DebugPickingPlugin>()
        )        
        .add_plugins(WorldInspectorPlugin::new())

        .add_plugins(ui::KartaUiPlugin)

        .add_plugins(vault::VaultPlugin)

        .add_plugins(graph::context::ContextPlugin)
        .add_plugins(graph::graph_cam::GraphCamPlugin)
        .add_plugins(modes::ModePlugin)

        .add_plugins(scene::scene::ScenePlugin)
        .add_plugins(scene::scene_cam::SceneCamPlugin)

        .run();
}








