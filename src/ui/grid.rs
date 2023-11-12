
use bevy::{
    prelude::*, 
    render::render_resource::{ShaderRef, AsBindGroup}, 
    pbr::{ExtendedMaterial, MaterialExtension}, sprite::{MaterialMesh2dBundle, Material2d, Material2dPlugin}
};

// Modeled after lib.rs of bevy_infinite_grid

pub struct InfiniteGrid2DPlugin;

impl Plugin for InfiniteGrid2DPlugin {
    fn build(&self, app: &mut App) {
        let mut material_plugin =
            // MaterialPlugin::<ExtendedMaterial<StandardMaterial, GridMaterial>>::default();
            Material2dPlugin::<GridMaterial>::default();
            // material_plugin.prepass_enabled = false;

        app
            .add_plugins(material_plugin)
            .insert_resource(InfiniteGrid2DSettings::default())

            .add_systems(Startup, setup_grid)
            
        ;
        
        
    }

    // fn finish(&self, app: &mut App) {
    //     render_app_builder(app);
    // }
}

fn setup_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut grid_materials: ResMut<Assets<ExtendedMaterial<ColorMaterial, GridMaterial>>>,
    mut grid_materials: ResMut<Assets<GridMaterial>>,
    // mut grid_materials: ResMut<Assets<ColorMaterial>>,
){
    commands.spawn((
        GraphBackground,
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Quad::new(Vec2::new(10000.0, 10000.0)).into()).into(),
            // material: grid_materials.add(ExtendedMaterial {
            //     base: StandardMaterial::from(Color::WHITE),
            //     extension: GridMaterial {
            //         cell_size: 1.0,
            //         cell_count: 10,
            //         color: Color::WHITE,
            //     },
            // }),

            material: grid_materials.add(GridMaterial {
                cell_size: 1.0,
                cell_count: 10,
                color: Color::ORANGE,
            }),

            // material: grid_materials.add(ColorMaterial::from(Color::ORANGE)),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, -1000.0),
                // rotation: Quat::from_rotation_x(std::f32::consts::PI / 2.0), // Rotates the quad to face along Z-axis
                ..default()
            },
            ..default()
        }
    ));
}


#[derive(Component)]
pub struct GraphBackground;

#[derive(Resource)]
pub struct InfiniteGrid2DSettings {
    pub cell_size: f32,
    pub cell_count: u32,
    pub color: Color,
}

impl Default for InfiniteGrid2DSettings {
    fn default() -> Self {
        InfiniteGrid2DSettings {
            cell_size: 1.0,
            cell_count: 10,
            color: Color::WHITE,
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GridMaterial {
    #[uniform(100)]
    pub cell_size: f32,
    #[uniform(101)]
    pub cell_count: u32,
    #[uniform(102)]
    pub color: Color,
}

impl Material2d for GridMaterial {
    fn fragment_shader() -> ShaderRef {
        "grid_material.wgsl".into()
    }
}

impl MaterialExtension for GridMaterial {
    fn fragment_shader() -> ShaderRef {
        "grid_material.wgsl".into()
    }
}