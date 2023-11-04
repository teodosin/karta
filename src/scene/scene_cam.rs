use bevy::prelude::{Camera3dBundle, Startup, App, Plugin, Commands, Camera, Transform, Vec3};
use bevy::utils::default;

pub struct SceneCamPlugin;

impl Plugin for SceneCamPlugin {
    fn build(&self, app: &mut App){
        app
            .add_systems(Startup, cam_setup)
        ;
    }
}

fn cam_setup(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        camera: Camera {
            order: 0,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        ..default()
    });
}