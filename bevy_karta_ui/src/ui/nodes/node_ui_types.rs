use bevy::{
    asset::{AssetServer, Assets, Handle},
    color::Color,
    core::Name,
    ecs::{entity::Entity, system::Commands},
    math::{self, Vec2, Vec3},
    prelude::default,
    render::{mesh::Mesh, texture::Image},
    sprite::{ColorMaterial, MaterialMesh2dBundle, Sprite, SpriteBundle},
    transform::components::Transform,
};
use rand::Rng;

use crate::{
    prelude::GraphEntity,
    ui::{graph_cam::ViewData, nodes::ViewNodeShape},
};

use super::{add_node_base_outline, add_node_label, TargetPosition};

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
    entity: Entity,
    data: &GraphEntity,
    name: Option<&Name>,
    spawn_pos: Vec2,
    tpos: Option<&TargetPosition>,

    mut commands: &mut Commands,

    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    view_data: &mut ViewData,
) {
    // Positions are slightly randomized to avoid nodes being spawned on top of each other
    let mut rng = rand::thread_rng();
    let label_pos = Vec2::new(40.0, 0.0);
    let radius = 35.0;

    let node_pos: Vec2 = match tpos {
        Some(pos) => pos.position,
        None => Vec2::new(
            spawn_pos.x + rng.gen_range(-10.0..10.0),
            spawn_pos.y + rng.gen_range(-10.0..10.0),
        ),
    };

    let node_z = view_data.get_z_for_node();

    println!("z depth for base node ui: {}", node_z);

    commands.entity(entity).insert((MaterialMesh2dBundle {
        mesh: meshes.add(math::primitives::Circle::new(radius)).into(),
        material: materials.add(ColorMaterial::from(Color::rgb(0.3, 0.0, 0.0))),
        transform: Transform::from_translation(Vec3::new(node_pos.x, node_pos.y, node_z)),
        ..default()
    },));

    if name.is_some() {
        add_node_label(&mut commands, &entity, &name.unwrap(), label_pos, &node_z);
    }
    add_node_base_outline(&mut commands, &entity, radius, &node_z);
}

// FOLDER/DIRECTORY NODE
// ----------------------------------------------------------------

// FILE NODE
// ----------------------------------------------------------------

// IMAGE NODE
// ----------------------------------------------------------------

// pub fn add_image_node_ui(
//     entity: Entity,
//     data: &GraphEntity,
//     spawn_pos: Vec2,
//     tpos: Option<&TargetPosition>,

//     mut commands: &mut Commands,

//     server: &mut AssetServer,
//     view_data: &mut ViewData,
// ){
//     let mut rng = rand::thread_rng();

//     let full_path = &data.path;
//     println!("Adding image node ui: {:#?}", full_path);

//     let accepted_image_formats = vec!["png", "jpg", "jpeg", "gif", "bmp", "tga", "tif", "tiff", "webp", "ico",];
//     match full_path.extension() {
//         Some(ext) => {
//             if !accepted_image_formats.contains(&ext.to_str().unwrap()) {
//                 return;
//             }
//             println!("Found extension: {}", ext.to_string_lossy());
//         },
//         None => return,
//     }

//     // let metadata = full_path.metadata().unwrap();
//     // println!("Metadata: {:#?}", metadata);

//     let image: Handle<Image> = server.load(full_path.clone());

//     let node_pos: Vec2 = match tpos {
//         Some(pos) => pos.position,
//         None => {
//             Vec2::new(
//                 spawn_pos.x + rng.gen_range(-10.0..10.0),
//                 spawn_pos.y + rng.gen_range(-10.0..10.0),
//             )
//         },
//     };

//     let node_z = view_data.get_z_for_node();
//     let size = Vec2::new(60.0, 40.0);

//     commands.entity(entity).insert((

//         // The actual dimensions of the image are not known until it is loaded.
//         // The dimensions are set in on_image_load() in asset_manager.rs
//         ViewNodeShape::Rectangle(size),
//         SpriteBundle {
//             texture: image,
//             sprite: Sprite {
//                 //anchor: bevy::sprite::Anchor::Center,
//                 ..default()
//             },
//             transform: Transform {
//                 translation: Vec3::new(
//                     node_pos.x,
//                     node_pos.y,
//                     node_z,
//                 ),
//                 ..default()
//             },
//             ..default()
//         },
//     ));

//     let pos = Vec2::new(-size.x / 2.0, -size.y / 2.0);

//     add_node_label(&mut commands, &entity, &data.path, pos, &node_z);
//     // add_node_rect_outline(&mut commands, &ev.entity, size, &view_data.top_z);
//     add_node_base_outline(&mut commands, &entity, size.x, &node_z);

// }

// TEXT NODE
// ----------------------------------------------------------------

// SVG NODE
// ----------------------------------------------------------------

pub fn add_svg_node_ui() {}

pub fn debug_sprite_picking() {}
