//lib

use bevy::{prelude::*, log::LogPlugin, window::WindowResolution};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::prelude::*;

mod tests;

mod vault;

mod actions;

mod events;

mod graph;
mod scene;

mod bevy_overlay_graph;



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
        .add_plugins(DefaultPickingPlugins
            .build()
            .disable::<BevyUiBackend>()
            // .disable::<DebugPickingPlugin>()
        )        

        .add_plugins(bevy_overlay_graph::OverlayGraphPlugin)

        // EGUI INSPECTOR BLOCK
        .add_plugins(WorldInspectorPlugin::new())

        // ENVIRONMENT BLOCK
        // The data that needs to remain in memory for the entire duration of the app
        .add_plugins(vault::VaultPlugin)// PreStartup
        
        // INPUT BLOCK
        // Plugins that handle input and interaction. 
        // Stage: PreUpdate

        // Actions that handle communication between input and the rest of the app.
        // Mostly PreUpdate. 
        .add_plugins(actions::ActionPlugin)
        .add_plugins(events::KartaEventPlugin)

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

        // .run()
    ;

    // let dot = bevy_mod_debugdump::schedule_graph_dot(&mut app, Startup, &bevy_mod_debugdump::schedule_graph::Settings::default());
    // std::fs::write("src/schedule_graphs/startup.dot", dot).expect("Unable to write file");

    // let dot = bevy_mod_debugdump::schedule_graph_dot(&mut app, PreUpdate, &bevy_mod_debugdump::schedule_graph::Settings::default());
    // std::fs::write("src/schedule_graphs/preupdate.dot", dot).expect("Unable to write file");

    // let dot = bevy_mod_debugdump::schedule_graph_dot(&mut app, Update, &bevy_mod_debugdump::schedule_graph::Settings::default());
    // std::fs::write("src/schedule_graphs/update.dot", dot).expect("Unable to write file");

    // let dot = bevy_mod_debugdump::schedule_graph_dot(&mut app, PostUpdate, &bevy_mod_debugdump::schedule_graph::Settings::default());
    // std::fs::write("src/schedule_graphs/postupdate.dot", dot).expect("Unable to write file");

    // let dot = bevy_mod_debugdump::schedule_graph_dot(&mut app, Last, &bevy_mod_debugdump::schedule_graph::Settings::default());
    // std::fs::write("src/schedule_graphs/last.dot", dot).expect("Unable to write file");

    app.run();
}








