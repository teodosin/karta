use bevy::{
    ecs::system::Commands, 
    asset::{Assets, AssetServer, Handle}, 
    render::{mesh::{Mesh, shape}, color::Color, texture::Image}, 
    sprite::{ColorMaterial, MaterialMesh2dBundle, SpriteBundle, Sprite}, 
    transform::components::Transform, math::{Vec3, Vec2}, 
    prelude::default, ui::FocusPolicy
};
use bevy_mod_picking::{PickableBundle, picking_core::Pickable, backends::raycast::RaycastPickable};
use rand::Rng;

use crate::{events::nodes::NodeSpawnedEvent, ui::graph_cam::ViewData};

use super::{add_node_label, add_node_base_outline};


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

    mut commands: &mut Commands,

    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    view_data: &mut ViewData,
){
    // Positions are slightly randomized to avoid nodes being spawned on top of each other
    let mut rng = rand::thread_rng();
    let label_pos = Vec2::new(35.0, 0.0);
    let radius = 35.0;

    let node_pos: Vec2 = match ev.rel_target_position {
        Some(pos) => ev.root_position + pos,
        None => {
            Vec2::new(
                ev.root_position.x + rng.gen_range(-10.0..10.0),
                ev.root_position.y + rng.gen_range(-10.0..10.0),
            )
        },
    };

    let node_z = view_data.get_z_for_node();

    println!("z depth for base node ui: {}", node_z);

    commands.entity(ev.entity).insert((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(radius).into()).into(),
            material: materials.add(ColorMaterial::from(Color::rgb(0.3, 0.0, 0.0))),
            transform: Transform::from_translation(Vec3::new(
                node_pos.x,
                node_pos.y,
                node_z,
            )),
            ..default()
        },
    ));

    add_node_label(&mut commands, &ev, label_pos, &node_z);
    add_node_base_outline(&mut commands, &ev.entity, radius, &node_z);
}

// FOLDER/DIRECTORY NODE
// ----------------------------------------------------------------

// FILE NODE
// ----------------------------------------------------------------

// IMAGE NODE
// ----------------------------------------------------------------

pub fn add_image_node_ui(
    ev: &NodeSpawnedEvent,
    // data: &Option<Box<dyn NodeData>>,

    mut commands: &mut Commands,

    server: &mut AssetServer,
    view_data: &mut ViewData,
){
    let mut rng = rand::thread_rng();

    let full_path = &ev.path;
    println!("Adding image node ui: {:?}", full_path);

    let accepted_image_formats = vec!["png", "jpg", "jpeg", "gif", "bmp", "tga", "tif", "tiff", "webp", "ico",];
    match full_path.extension() {
        Some(ext) => {
            if !accepted_image_formats.contains(&ext.to_str().unwrap()) {
                return;
            }
            println!("Found extension: {}", ext.to_string_lossy());
        },
        None => return,
    }

    // let metadata = full_path.metadata().unwrap();
    // println!("Metadata: {:?}", metadata);

    let image: Handle<Image> = server.load(full_path.clone());

    println!("Position: {:?}", ev.root_position);

    let node_pos: Vec2 = match ev.rel_target_position {
        Some(pos) => ev.root_position + pos,
        None => {
            Vec2::new(
                ev.root_position.x + rng.gen_range(-10.0..10.0),
                ev.root_position.y + rng.gen_range(-10.0..10.0),
            )
        },
    };

    let node_z = view_data.get_z_for_node();

    commands.entity(ev.entity).insert((

        SpriteBundle {
            texture: image,
            sprite: Sprite {
                //anchor: bevy::sprite::Anchor::Center,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(
                    node_pos.x,
                    node_pos.y,
                    node_z,
                ),
                ..default()
            },
            ..default()
        },
    ));

    let size = Vec2::new(60.0, 40.0);
    let pos = Vec2::new(-size.x / 2.0, -size.y / 2.0);

    add_node_label(&mut commands, &ev, pos, &node_z);
    // add_node_rect_outline(&mut commands, &ev.entity, size, &view_data.top_z);
    add_node_base_outline(&mut commands, &ev.entity, size.x, &node_z);

}
                        
// TEXT NODE
// ----------------------------------------------------------------

// SVG NODE
// ----------------------------------------------------------------

pub fn add_svg_node_ui(

){

}