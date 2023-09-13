/// Main file for the Context plugin
/// The Context manages the node graph 
/// 

use bevy::{prelude::*, utils::HashMap, sprite::MaterialMesh2dBundle, text::Text2dBounds, ecs::{query, event}, transform::commands};
use bevy_mod_picking::prelude::*;
use rand::Rng;
use std::fs;

use crate::{MoveNodesEvent, Selected, GraphNode, GraphEdge, ViewData, InputData};

pub struct ContextPlugin;

impl Plugin for ContextPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(PathsToEntitiesIndex(HashMap::new()))
            .insert_resource(KartaVault::new())

            .add_event::<NodeInputEvent>()

            .add_systems(Startup, gizmo_settings)
            .add_systems(Startup, initial_context)

            .add_systems(PreUpdate, handle_node_click)

            .add_systems(Update, change_context)
            .add_systems(Update, despawn_nodes.after(change_context))

            .add_systems(PreUpdate, draw_edges);
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
    current_context: String,
}

impl KartaVault {
    fn new() -> Self {
        KartaVault {
            vault_folder_name: "kartaVault".to_string(),
            root: "home/viktor/Pictures".to_string(),
            current_context: "home/viktor/Pictures".to_string(),
        }
    }

    fn get_root_path(&self) -> String {
        format!("/{}", self.root)
    }

    fn get_vault_path(&self) -> String {
        format!("/{}/{}", self.root, self.vault_folder_name)
    }

    fn set_current_context(&mut self, path: String) {
        self.current_context = path;
    }

    fn get_current_context_path(&self) -> String {
        format!("/{}", self.current_context)
    }
}

fn gizmo_settings(
    mut gizmo: ResMut<GizmoConfig>,
){
    gizmo.depth_bias = 1.0;
}

fn draw_edges(
    edges: Query<&GraphEdge>,
    nodes: Query<&Transform, With<GraphNode>>,
    _cameras: Query<&Camera>,
    mut gizmos: Gizmos,
) {
    for edge in edges.iter() {
        let start = match nodes.get(edge.from) {
            Ok(node) => node,
            Err(_) => continue,
        };
        let end = match nodes.get(edge.to){
            Ok(node) => node,
            Err(_) => continue,
        };
        gizmos.line_2d(
            Vec2::new(start.translation.x, start.translation.y),
            Vec2::new(end.translation.x, end.translation.y),
            Color::GREEN,
        );
    }
}

fn initial_context(
    mut event: EventWriter<NodeInputEvent>,
){
    event.send(NodeInputEvent {
        target: None,
    });
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

fn change_context(
    event: EventReader<NodeInputEvent>,
    input_data: Res<InputData>,

    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,

    mut vault: ResMut<KartaVault>,
    mut view_data: ResMut<ViewData>,
    mut pe_index: ResMut<PathsToEntitiesIndex>,

    mut young_nodes: Query<(Entity, &GraphNode, Option<&JustSpawned>)>,
    mut nodes: Query<(Entity, &GraphNode)>,
) {
    // Only run the system if there has been a node input
    if event.is_empty(){
        return
    }

    
    // Handle the path to the desired context
    let path: String = input_data.latest_target_entity.clone()
    .unwrap_or(vault.get_current_context_path());
    // Also return if the target path is already the current context
    if path == vault.get_current_context_path() && path != vault.get_root_path(){
        println!("Already in context: {}", path);
        return
    }

    println!("Path: {}", path);
    let entries = fs::read_dir(&path);

    // Get all file and folder names in 
    let entries = match entries {
        Ok(entries) => entries,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };

    // Get all files
    let file_names: Vec<String> = entries
    .filter_map(|entry| {
        let path = entry.ok()?.path();
        if path.is_file() {
            path.file_name()?.to_str().map(|s| s.to_owned())
        } else {
            path.file_name()?.to_str().map(|s| s.to_owned())
        }
    })
    .collect();

    // Iterate through existing nodes and mark them for deletion
    for (entity, node) in nodes.iter_mut() {
        commands.entity(entity).insert(ToBeDespawned);
    }

    // Spawn the context root if it doesn't exist
    let root_node = match pe_index.0.get(&path) {
        Some(entity) => {
            println!("Root node already exists");
            commands.entity(*entity).remove::<ToBeDespawned>();
            *entity
            
        },
        None => {
            println!("Root node doesn't exist, spawning");
            let root_name = path.split("/").last().unwrap().to_string();
            let root_path = path.replace(&root_name, "");
            let root_path = &root_path[0..&root_path.len()-1].to_string();
            println!("Root Path: {}, Root Name: {}", root_path, root_name);
            spawn_node(
                &mut commands, 
                &root_path, 
                &root_name,
                &mut meshes, 
                &mut materials, 
                &mut view_data,
                &mut pe_index,
            )
        }
    };

    // Don't despawn the parent of the root
    let root_parent_path = path
        .replace(&path.split("/")
        .last()
        .unwrap(), "");
    let root_parent_path = &root_parent_path[0..&root_parent_path.len()-1].to_string();
    let root_parent = pe_index.0.get(root_parent_path);
    println!("Root parent: {:?}", root_parent_path);
    match root_parent {
        Some(entity) => {
            commands.entity(*entity).remove::<ToBeDespawned>();
        },
        None => {
            println!("Root parent doesn't exist");
        }
    }
    
    file_names.iter().for_each(|name| {

        // Check if the item already exists
        let full_path = format!("{}/{}", path, name);
        let item_exists = pe_index.0.get(&full_path).is_some();
        if item_exists {
                println!("Item already exists: {}", full_path);
                // Remove despawn component
                commands.entity(pe_index.0.get(&full_path).unwrap().clone()).remove::<ToBeDespawned>();
                return
        }

        // Spawn a node for each item
        let node = spawn_node(
            &mut commands,
            &path,
            name,
            &mut meshes,
            &mut materials,
            &mut view_data,
            &mut pe_index,
        );

        // Spawn an edge from the root node to each item
        commands.spawn((GraphEdge {
            from: root_node,
            to: node,
            attributes: vec![],
        },));

        // let image_handle: Handle<Image> =_asset_server.load::<Image, String>(full_path);

        // let image_size = assets.get(&image_handle);
        // match image_size {
        //     Some(image_size) => {
        //         println!("Image size: {},{}", 
        //             image_size.texture_descriptor.size.width, 
        //             image_size.texture_descriptor.size.height
        //         );
        //     },
        //     None => {
        //         println!("Image size: None");
        //     }
        // }
    });

    vault.set_current_context(path.clone());

    // Print pe_index to see what the hell is going on
    for (path, entity) in pe_index.0.iter() {
        println!("Path: {}, Entity: {:?}", path, entity);
    };
}

fn despawn_nodes(
    mut commands: Commands,
    mut nodes: Query<(Entity, &GraphNode), With<ToBeDespawned>>,
    mut pe_index: ResMut<PathsToEntitiesIndex>,
){
    for (entity, node) in nodes.iter_mut() {
        commands.entity(entity).despawn_recursive();
        pe_index.0.remove(&node.path);
    }
}

fn handle_node_click(
   mut event: EventReader<NodeInputEvent>,
    mut input_data: ResMut<InputData>,
    nodes: Query<&GraphNode>,
){
    if event.is_empty() {
        return
    }
    match event.iter().next().unwrap().target {
        None => {
            println!("No event");
            input_data.latest_target_entity = None;
        }
        Some(target) => {
            println!("Event: {:?}", target);
            let target_path = nodes.get(target).unwrap().path.clone();
        
            input_data.latest_target_entity = Some(target_path.clone());
            println!("Target path: {}", target_path);
        },
    }
}

#[derive(Event)]
struct NodeInputEvent {
    target: Option<Entity>,
}

impl From<ListenerInput<Pointer<Click>>> for NodeInputEvent {
    fn from(event: ListenerInput<Pointer<Click>>) -> Self {
        NodeInputEvent {
            target: Some(event.target),
        }
    }
}

#[derive(Component)]
struct JustSpawned;

#[derive(Component)]
struct ToBeDespawned;

fn spawn_node (
    commands: &mut Commands,
    path: &String,
    name: &String,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    view_data: &mut ResMut<ViewData>,
    pe_index: &mut ResMut<PathsToEntitiesIndex>,
) -> bevy::prelude::Entity {
    let full_path = format!("{}/{}", path, name);

    // Positions will be random for now
    let mut rng = rand::thread_rng();

    let node = commands.spawn((
        GraphNode {
            path: full_path.clone()
        },
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(25.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::rgb(0.3, 0.0, 0.0))),
            transform: Transform::from_translation(Vec3::new(
                rng.gen_range(-600.0..600.0),
                rng.gen_range(-400.0..400.0),
                view_data.top_z,
            )),
            ..default()
        },
        PickableBundle::default(),
        RaycastPickTarget::default(),

        On::<Pointer<Drag>>::send_event::<MoveNodesEvent>(),
        On::<Pointer<Click>>::send_event::<NodeInputEvent>(),
        // On::<Pointer<Drag>>::run(move_node_selection),
        On::<Pointer<DragStart>>::target_insert(Selected),
        On::<Pointer<DragEnd>>::target_remove::<Selected>(),
        On::<Pointer<Deselect>>::target_remove::<Selected>(),

        JustSpawned,
    )).id();
        
    let name_label = commands.spawn(Text2dBundle {
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
    }).id();

    commands.entity(node).push_children(&[name_label]);
        
            // Commented out for now. 
            // Spawning nodes is generic until I figure out
            // how to handle different types of nodes. 

            // parent.spawn((SpriteBundle {
            //     texture: image_handle,
            //     sprite: Sprite {
            //         anchor: bevy::sprite::Anchor::TopLeft,
            //         ..default()
            //     },
                
            //     transform: Transform {
            //         translation: Vec3::new(0.0, -30.0, 0.1),
            //         scale: Vec3::new(1.0, 1.0, 0.05),
            //         ..default()
            //     },
            //     ..default()
            // },
            // PickableBundle {
            //     pickable: Pickable {
            //         should_block_lower: false,
            //         ..default()
            //     },
            //     ..default()
            // }
        // ));        


    // Update the view_data so we can keep track of which zindex is the topmost
    view_data.top_z += 1.;

    // Update the PathsToEntitiesIndex
    pe_index.0.insert(full_path, node);

    // Return the node entity
    node
    
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

