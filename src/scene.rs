use bevy::prelude::{Plugin, App};

pub mod scene;
pub mod scene_cam;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_plugins(scene::ScenePlugin)
            .add_plugins(scene_cam::SceneCamPlugin)
        ;
    }
}