/// Main file for the Context plugin
/// The Context manages the node graph 
/// 

use bevy::{prelude::*, utils::HashMap, sprite::MaterialMesh2dBundle, text::Text2dBounds};
use bevy_mod_picking::prelude::*;
use rand::Rng;
use std::{fs, io};

use crate::{MoveNodesEvent, Selected, GraphNode};

pub struct ContextPlugin;

impl Plugin for ContextPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(PathsToEntitiesIndex(HashMap::new()))
            .insert_resource(KartaVault::new())

            .add_systems(Startup, spawn_from_context);
    }
}

#[derive(Resource, Debug)]
struct PathsToEntitiesIndex(
    HashMap<String, Entity>,
);

#[derive(Resource, Debug)]
struct KartaVault{
    vault_folder_name: String,
    root: String,
}

impl KartaVault {
    fn new() -> Self {
        KartaVault {
            vault_folder_name: "kartaVault".to_string(),
            root: "home".to_string(),
        }
    }

    fn get_root_path(&self) -> String {
        format!("/{}", self.root)
    }

    fn get_vault_path(&self) -> String {
        format!("/{}/{}", self.root, self.vault_folder_name)
    }
}

// Spawn and despawn functions

// Spawn the context of a path
// Take in the path of a node
// Take in an optional filter 
// Check if the path has a corresponding context file
// If it does, spawn the nodes and edges in the context file
// If the path is a folder, spawn the contents of the folder as nodes and edges
// If the path is a file, spawn the file as a node
// Check the nodes against the filter, don't spawn if it doesn't match
fn spawn_from_context(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    _asset_server: Res<AssetServer>,
    vault: Res<KartaVault>,
) {
    let path = vault.get_root_path();
    let path = "/home/viktor/Downloads".to_owned();
    println!("Path: {}", path);
    let entries = fs::read_dir(path);

    let entries = match entries {
        Ok(entries) => entries,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };

    let file_names: Vec<String> = entries
    .filter_map(|entry| {
        let path = entry.ok()?.path();
        if path.is_file() {
            path.file_name()?.to_str().map(|s| s.to_owned())
        } else {
            None
        }
    })
    .collect();

    file_names.iter().enumerate().for_each(|(i, name)| {
        let mut rng = rand::thread_rng();
    
        commands.spawn((
            GraphNode,
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(25.).into()).into(),
                material: materials.add(ColorMaterial::from(Color::rgb(0.3, 0.0, 0.0))),
                transform: Transform::from_translation(Vec3::new(
                    rng.gen_range(-600.0..600.0),
                    rng.gen_range(-400.0..400.0),
                    i as f32,
                )),
                ..default()
            },
            PickableBundle::default(),
            RaycastPickTarget::default(),
    
            On::<Pointer<Drag>>::send_event::<MoveNodesEvent>(),
            On::<Pointer<DragStart>>::target_insert(Selected),
            On::<Pointer<Deselect>>::target_remove::<Selected>(),
        ))
            .with_children(|parent| {
                parent.spawn(Text2dBundle {
                    text: Text {
                        sections: vec![TextSection::new(
                            name,
                            TextStyle {
                                font_size: 20.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        )],
                        ..default()
                    },
                    text_2d_bounds: Text2dBounds {
                        size: Vec2::new(400.0, 200.0),
                        ..default()
                    },
                    transform: Transform {
                        translation: Vec3::new(30.0, 0.0, 0.1),
                        ..default()
                    },
                    text_anchor: bevy::sprite::Anchor::CenterLeft,
                    ..default()
                });
            });
    });
}
// Do the reverse of the above for the despawn function


// Collapse and expand functions

// Similar to the spawn functions, but manages aliases also 
// So that when a node group is collapsed, it is replaced by its alias edge
// The edge that pointed to that node now points to the alias edge

// If a node group is expanded, the alias edge is replaced by the node group
// and their relevant edges.
// If an individual node is expanded and its file format is supported,
// its contents and their relevant edges are spawned around it (or in it)

