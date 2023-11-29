//lib

use bevy::{prelude::*, log::LogPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::prelude::*;


mod vault;
mod settings;

mod modes;
mod input;
mod actions;

mod events;

mod graph;
mod scene;

mod ui;



pub fn karta_app() {
    let mut app = App::new();

    app
        // PLUGIN BLOCK
        .add_plugins(DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Karta".to_string(),
                    ..default()
                }),
                ..default()
            })
            .build()
            .disable::<LogPlugin>()
        )
        .add_plugins(DefaultPickingPlugins
            .build()
            .disable::<DebugPickingPlugin>()
        )        

        // EGUI INSPECTOR BLOCK
        .add_plugins(WorldInspectorPlugin::new())

        // ENVIRONMENT BLOCK
        // The data that needs to remain in memory for the entire duration of the app
        .add_plugins(vault::VaultPlugin)// PreStartup
        .add_plugins(settings::SettingsPlugin)
        
        // INPUT BLOCK
        // Plugins that handle input and interaction. 
        // Stage: PreUpdate
        .add_plugins(modes::ModePlugin)
        .add_plugins(input::InputPlugin)

        // Actions that handle communication between input and the rest of the app.
        // Mostly PreUpdate. 
        .add_plugins(actions::ActionPlugin)
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

        //.run()

        // It's been brought to my attention that my way of structuring my plugins might suck a bit.
        // My plugins are strongly coupled to each other. I can't test plugins in isolation.
        // I can create test cases still, and import everything I need to test a plugin, but it's 
        // not ideal. So, TODO!
        // Events and actions are prime candidates to be dissolved into other plugins.
        // I am not confident that the others can be dissolved.
    ;

    // let dot = bevy_mod_debugdump::schedule_graph_dot(&mut app, Update, &bevy_mod_debugdump::schedule_graph::Settings::default());
    // // Save dot output as a file
    // std::fs::write("strings_notworking_update.dot", dot).expect("Unable to write file");

    // let dot = bevy_mod_debugdump::schedule_graph_dot(&mut app, PostUpdate, &bevy_mod_debugdump::schedule_graph::Settings::default());
    // // Save dot output as a file
    // std::fs::write("strings_notworking_postupdate.dot", dot).expect("Unable to write file");

    app.run();
}








