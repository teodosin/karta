use bevy::{ecs::{system::{Resource, SystemId, Commands, ResMut}, world::World, event::EventReader}, utils::HashMap, asset::{Assets, AssetServer}, render::{mesh::{Mesh, shape}, color::Color}, sprite::{ColorMaterial, MaterialMesh2dBundle, SpriteBundle, Sprite}, transform::components::Transform, math::Vec3, prelude::default};
use rand::Rng;

use crate::{graph::{node_types::{NodeTypes, DataTypes, NodeData}, graph_cam::ViewData}, events::nodes::NodeSpawnedEvent};


// TODO: Convert back to using one-shot systems in 0.13
// #[derive(Resource)]
// pub struct UiNodeSystemsIndex {
//     pub systems: HashMap<NodeTypes, SystemId>
// }

// impl Default for UiNodeSystemsIndex {
//     fn default() -> Self {
//         UiNodeSystemsIndex {
//             systems: HashMap::new(),
//         }
//     }
// }

// pub fn setup_node_ui_systems(
//     world: &mut World,
// ){
//     let mut index = world.get_resource_mut::<UiNodeSystemsIndex>().unwrap();

//     index.systems.insert(NodeTypes::Base, world.register_system(add_base_node_ui));
    
// }


// BASE NODE
// ----------------------------------------------------------------
// For the node types that don't have a specific ui 

pub fn add_base_node_ui(
    ev: &NodeSpawnedEvent,

    commands: &mut Commands,

    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    view_data: &mut ViewData,
){
    // Positions are slightly randomized to avoid nodes being spawned on top of each other
    let mut rng = rand::thread_rng();
    let radius = 25.0;

    commands.entity(ev.entity).insert((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(radius).into()).into(),
            material: materials.add(ColorMaterial::from(Color::rgb(0.3, 0.0, 0.0))),
            transform: Transform::from_translation(Vec3::new(
                ev.position.x + rng.gen_range(-10.0..10.0),
                ev.position.y + rng.gen_range(-10.0..10.0),
                view_data.top_z,
            )),
            ..default()
        },
    ));
    // Update the view_data so we can keep track of which zindex is the topmost
    view_data.top_z += 0.0001;
}

// FOLDER/DIRECTORY NODE
// ----------------------------------------------------------------

// FILE NODE
// ----------------------------------------------------------------

// IMAGE NODE
// ----------------------------------------------------------------

pub fn add_image_node_ui(
    ev: &NodeSpawnedEvent,
    data: &Option<Box<dyn NodeData>>,

    commands: &mut Commands,

    server: &mut AssetServer,
    view_data: &mut ViewData,
){
    let mut rng = rand::thread_rng();

    let full_path = ev.path.join(&ev.name);
    println!("Adding image node ui: {:?}", full_path);

    let accepted_image_formats = vec!["png", "jpg", "jpeg", "gif", "bmp", "tga", "tif", "tiff", "webp", "ico",];
    match full_path.extension() {
        Some(ext) => {
            if !accepted_image_formats.contains(&ext.to_str().unwrap()) {
                return;
            }
        },
        None => return,
    }

    let metadata = full_path.metadata().unwrap();
    println!("Metadata: {:?}", metadata);

    let image = server.load(full_path.clone());

    println!("Position: {:?}", ev.position);

    commands.entity(ev.entity).insert((
        SpriteBundle {
            texture: image,
            sprite: Sprite {
                anchor: bevy::sprite::Anchor::Center,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(
                    ev.position.x + rng.gen_range(-10.0..10.0),
                    ev.position.y + rng.gen_range(-10.0..10.0),
                    view_data.top_z,
                ),
                ..default()
            },
            ..default()
        },
    ));

    view_data.top_z += 0.0001;
}
                        
// TEXT NODE
// ----------------------------------------------------------------

// SVG NODE
// ----------------------------------------------------------------

pub fn add_svg_node_ui(

){

}