
use bevy::{
    math, prelude::*, render::{render_resource::{AsBindGroup, ShaderRef}, view::RenderLayers}, sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle}
};
use bevy_mod_picking::{prelude::*, backends::raycast::RaycastPickable};

use super::graph_cam::{GraphCamera, graph_zoom};
// Modeled after lib.rs of bevy_infinite_grid


pub struct InfiniteGrid2DPlugin;

impl Plugin for InfiniteGrid2DPlugin {
    fn build(&self, app: &mut App) {
        let material_plugin =
            Material2dPlugin::<GridMaterial>::default();
            
        app
            .add_plugins(material_plugin)
            .add_systems(PreStartup, setup_grid)  
            .add_systems(Update, update_grid_zoom.after(graph_zoom)) 
        ;
    }
}

fn setup_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut grid_materials: ResMut<Assets<GridMaterial>>,
){
    commands.spawn((
        RenderLayers::layer(31),
        GraphBackground,
        MaterialMesh2dBundle {
            mesh: meshes.add(math::primitives::Rectangle::new(1000000.0, 1000000.0)).into(),
            material: grid_materials.add(GridMaterial::default()),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, -1000.0),
                ..default()
            },
            ..default()
        },
        RaycastPickable,
        Pickable {
            is_hoverable: true,
            should_block_lower: true,
        },

    ));
}

fn update_grid_zoom(
    mut grid_material: ResMut<Assets<GridMaterial>>,
    grid: Query<&Handle<GridMaterial>, With<GraphBackground>>,
    mut camera: Query<&OrthographicProjection, (Changed<OrthographicProjection>, With<GraphCamera>)>, 
){
    for projection in camera.iter_mut() {
        let zoom = projection.scale;
        for handle in grid.iter() {
            if let Some(material) = grid_material.get_mut(handle) {
                material.zoom = zoom;
                println!("Grid zoom: {}", zoom);
            }
        }
    }
}



#[derive(Component)]
pub struct GraphBackground;


#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GridMaterial {
    #[uniform(0)]
    pub zoom: f32,
    #[uniform(1)]
    pub color: LinearRgba,
    #[uniform(2)]
    pub grid_cell_size: Vec2,
}

impl Default for GridMaterial {
    fn default() -> Self {
        GridMaterial {
            zoom: 1.0,
            color: LinearRgba::new(1.0, 1.0, 1.0, 0.2),
            grid_cell_size: Vec2::new(0.00004, 0.00004),
        }
    }
}

impl Material2d for GridMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://bevy_overlay_graph/assets/grid_material.wgsl".into()
    }
}