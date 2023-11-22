use bevy::{prelude::{Plugin, App}, app::Update};

use self::engine::evaluate_active_graph;

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