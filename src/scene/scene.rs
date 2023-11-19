// Management of the currently active 3D scene

use std::{f32::consts::PI, path::PathBuf};

use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::graph::context::CurrentContext;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(CurrentActive { active: None })

            .add_systems(Startup, spawn_some_spheres)

            .add_systems(Update, update_active_on_context_change.run_if(resource_changed::<CurrentContext>()))
            .add_systems(Update, rotate);
    }
}

#[derive(Resource, Debug)]
pub struct CurrentActive {
    pub active: Option<PathBuf>,
}

fn update_active_on_context_change(
    context: Res<CurrentContext>,
    mut active: ResMut<CurrentActive>,
){
    let cxt = match &context.cxt {
        Some(cxt) => cxt,
        None => {
            println!("No context set");
            return
        }
    };
    
    active.active = Some(cxt.get_current_context_path());

}

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
struct Shape;

const X_EXTENT: f32 = 14.;

fn spawn_some_spheres(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    let shapes = [
        meshes.add(shape::Cube::default().into()),
        meshes.add(shape::Box::default().into()),
        meshes.add(shape::Capsule::default().into()),
        meshes.add(shape::Torus::default().into()),
        meshes.add(shape::UVSphere::default().into()),
    ];

    let num_shapes = shapes.len();

    for (i, shape) in shapes.into_iter().enumerate() {
        commands.spawn((
            PbrBundle {
                mesh: shape,
                material: debug_material.clone(),
                transform: Transform::from_xyz(
                    -X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * X_EXTENT,
                    2.0,
                    0.0,
                )
                .with_rotation(Quat::from_rotation_x(-PI / 4.)),
                ..default()
            },
            Shape,
        ));
    }

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });

    // ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane { size: 50.0, subdivisions: 2 }.into()),
        material: materials.add(Color::SILVER.into()),
        ..default()
    });


}

fn rotate(
    mut query: Query<&mut Transform, With<Shape>>, 
    time: Res<Time>,
    mut cam: Query<&mut Transform, (With<Camera3d>, Without<Shape>)>,
) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_seconds() / 2.);
    }

    for mut camera in &mut cam {
        // camera.rotate_y(time.delta_seconds() / 80.);
        let rotation = time.delta_seconds() / 40.;
        camera.translate_around(
            Vec3::ZERO, 
            Quat::from_rotation_y(rotation));
        camera.rotate_y(rotation);
        break
    }
}




fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
    )
}