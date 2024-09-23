// Shader taken from https://github.com/johnbchron/neutron
// Credit belongs to the author, permission pending.
// Should permission be denied, this shader will be removed.

use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};

#[derive(AsBindGroup, Debug, Clone, Asset, Reflect)]
pub struct CurveMaterial {
    #[uniform(0)]
    pub point_a: Vec2,
    #[uniform(0)]
    pub point_b: Vec2,
    #[uniform(0)]
    pub point_c: Vec2,
    #[uniform(0)]
    pub point_d: Vec2,
    #[uniform(0)]
    pub color: LinearRgba,
    #[uniform(0)]
    pub width: f32,
}

impl Default for CurveMaterial {
    fn default() -> Self {
        Self {
            point_a: Vec2::new(-160.0, 120.0),
            point_b: Vec2::new(-100.0, 120.0),
            point_c: Vec2::new(100.0, -120.0),
            point_d: Vec2::new(160.0, -120.0),
            color: LinearRgba::WHITE,
            width: 1.0,
        }
    }
}

impl Material2d for CurveMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/curve_material.wgsl".into()
    }
    fn vertex_shader() -> ShaderRef {
        "shaders/curve_material.wgsl".into()
    }
}

pub struct CurveMaterialPlugin;

impl Plugin for CurveMaterialPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(Material2dPlugin::<CurveMaterial>::default())
            .register_asset_reflect::<CurveMaterial>()
            .add_systems(Startup, test_curve_material)
        ;
    }
}

fn test_curve_material(
    mut commands: Commands,
    mut materials: ResMut<Assets<CurveMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((MaterialMesh2dBundle {
        mesh: meshes.add(Rectangle::default()).into(),
        transform: Transform::default().with_scale(Vec3::splat(128.)),
        material: materials.add(CurveMaterial {
            point_a: Vec2::new(-1.6, 1.2),
            point_b: Vec2::new(1.0, 1.2),
            point_c: Vec2::new(-1.0, -1.0),
            point_d: Vec2::new(1.6, -1.2),
            color: LinearRgba::rgb(0.3, 0.1, 0.3),
            width: 0.09,
        }),
        ..default()
    }, Name::new("curve_material")
    ));

    commands.spawn(Camera2dBundle::default());
}

// fn show_curve_material(
//     mut query: Query<&mut CurveMaterial>,
//     mut gizmos: Gizmos,

// ){

// }