use bevy::prelude::{Plugin, App};


pub mod scene;
pub mod engine;
pub mod scene_cam;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_plugins(scene::ScenePlugin)
            .add_plugins(scene_cam::SceneCamPlugin)

            //.add_systems(Update, evaluate_active_graph)
        ;
    }
}

/// Idea:
/// If the graph were a crate that could be added to any Bevy app, then the graph could be used to
/// visualise entity relationships and scene structures, for example. How all this works ergonomically
/// is to be determined, but perhaps users would want to mark entities that they want to include in the 
/// graph. So perhaps a marker component. 
#[derive(bevy::ecs::component::Component)]
pub struct GraphEntity;