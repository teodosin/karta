//lib

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::prelude::*;


mod vault;

mod modes;
mod input;
mod actions;
mod history;

mod events;

mod graph;
mod scene;

mod ui;



pub fn karta_app() {
    App::new()
    // PLUGIN BLOCK
    .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
    .add_plugins(DefaultPickingPlugins
        .build()
        .disable::<DebugPickingPlugin>()
    )        
        //.add_plugins(WorldInspectorPlugin::new())

    // ENVIRONMENT BLOCK
    // The data that needs to remain in memory for the entire duration of the app
    .add_plugins(vault::VaultPlugin)// PreStartup
    
    // INPUT BLOCK
    // Plugins that handle input and interaction. 
    // Stage: PreUpdate
    .add_plugins(modes::ModePlugin)
    .add_plugins(input::InputPlugin)

    // Actions that handle communication between input and the rest of the app.
    // Mostly PreUpdate. 
    .add_plugins(actions::ActionPlugin)
    .add_plugins(history::HistoryPlugin)
    .add_plugins(events::EventPlugin)

    // GRAPH BLOCK
    // Handle update to the graph. Evaluate operator graph. 
    .add_plugins(graph::GraphPlugin)
    
    // SCENE BLOCK
    // Handle update to the scene.
    .add_plugins(scene::ScenePlugin)

    // UI BLOCK
    // Handle update to the UI.
    // UI input is handled in PreUpdate
    // Drawing is done PostUpdate
    .add_plugins(ui::KartaUiPlugin)

    .run();
}








