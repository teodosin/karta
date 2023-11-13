
use bevy::{
    prelude::*, 
    render::render_resource::{ShaderRef, AsBindGroup}, 
    sprite::{MaterialMesh2dBundle, Material2d, Material2dPlugin}
};
use bevy_mod_picking::prelude::*;

use crate::events::background::{RectangleSelectionEvent, RectangleSelectionEndEvent};

// Modeled after lib.rs of bevy_infinite_grid

pub struct InfiniteGrid2DPlugin;

impl Plugin for InfiniteGrid2DPlugin {
    fn build(&self, app: &mut App) {
        let material_plugin =
            Material2dPlugin::<GridMaterial>::default();
        app
            .add_plugins(material_plugin)
            .add_systems(Startup, setup_grid)   
        ;
    }
}

fn setup_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut grid_materials: ResMut<Assets<GridMaterial>>,
){
    commands.spawn((
        GraphBackground,
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Quad::new(Vec2::new(10000.0, 10000.0)).into()).into(),
            material: grid_materials.add(GridMaterial::default().into()),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, -1000.0),
                ..default()
            },
            ..default()
        },

        // On::<Pointer<DragEnd>>::send_event::<RectangleSelectionEndEvent>(),
    ));
}


#[derive(Component)]
pub struct GraphBackground;

impl Default for GridMaterial {
    fn default() -> Self {
        GridMaterial {
            color: Color::ORANGE,
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GridMaterial {
    #[uniform(0)]
    pub color: Color,
}

impl Material2d for GridMaterial {
    fn fragment_shader() -> ShaderRef {
        "grid_material.wgsl".into()
    }
}