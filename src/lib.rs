//lib

use bevy::{input::mouse::*, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::prelude::*;

pub mod tools;

mod context;
mod scene;

use crate::context::ContextPlugin;
use crate::scene::ScenePlugin;

pub fn karta_app() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(DefaultPickingPlugins
            .build()
            .disable::<DebugPickingPlugin>()
        )        
        //.add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())

        .add_plugins(ContextPlugin)
        .add_plugins(ScenePlugin)

        .insert_resource(ViewSettings::default())
        .insert_resource(ViewData::default())
        .insert_resource(InputData::default())

        .add_systems(Startup, setup)

        .add_event::<MoveNodesEvent>()

        .add_systems(Update, move_node_selection)
        
        .add_systems(Update, graph_zoom)
        .add_systems(Update, graph_pan)

        .add_systems(PreUpdate, update_cursor_info)

        .run();
}

#[derive(Resource, Debug)]
struct ViewSettings {
    pub zoom: f32,
}

impl Default for ViewSettings {
    fn default() -> Self {
        ViewSettings { zoom: 1.0 }
    }
}

#[derive(Resource, Debug)]
pub struct ViewData {
    top_z: f32,
}

impl Default for ViewData {
    fn default() -> Self {
        ViewData { top_z: 0.0 }
    }
}


#[derive(Resource, Debug)]
struct InputData {
    pub latest_target_entity: Option<String>,
    pub drag_position: Vec2,
    pub prev_position: Vec2,
    pub curr_position: Vec2,
}

impl Default for InputData {
    fn default() -> Self {
        InputData {
            latest_target_entity: None,
            drag_position: Vec2::ZERO,
            prev_position: Vec2::ZERO,
            curr_position: Vec2::ZERO,
        }
    }
}

fn update_cursor_info(
    mut cursor_history: ResMut<InputData>,
    mouse: Res<Input<MouseButton>>,
    window: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    let (camera, camera_transform) = camera_q.single();

    cursor_history.prev_position = cursor_history.curr_position;

    if mouse.just_pressed(MouseButton::Middle) {
        cursor_history.drag_position = cursor_history.curr_position;
    }

    if let Some(world_position) = window.single().cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        cursor_history.curr_position = world_position;
    }
}

#[derive(Resource, Default, Debug)]
struct InputSettings {}

#[derive(Component, Clone)]
struct Selected;

#[derive(Component)]
pub struct GraphNode {
    pub path: String,
}

#[derive(Component)]
pub struct GraphNodeEdges {
    pub edges: Vec<Entity>,
}

#[derive(Component)]
pub struct GraphEdge {
    pub from: Entity,
    pub to: Entity,
    pub attributes: Vec<GraphEdgeAttribute>,
}

pub struct GraphEdgeAttribute {
    pub name: String,
    pub value: f32,
}

#[derive(Component)]
struct GraphPosition(Vec3);

impl GraphNode {
    fn screamies(&self, num: &f32) {
        println!("{}", num);
    }
}
#[derive(Component)]
struct GraphColor(Color);

fn setup(mut commands: Commands) {
    use bevy::core_pipeline::clear_color::ClearColorConfig;
    commands.spawn((
        Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::None,
                ..default()
            },
            camera: Camera {
                order: 1,
                ..default()
            },
            ..default()
        },
        RaycastPickCamera::default(),
    ));
    commands.spawn(Camera3dBundle {
        camera: Camera {
            order: 0,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        ..default()
    });
}


fn move_node_selection(
    mut ev_mouse_drag: EventReader<MoveNodesEvent>,
    mouse: Res<Input<MouseButton>>,
    cursor: Res<InputData>,
    mut query: Query<(Entity, &GraphNode, &mut Transform), With<Selected>>,
    mut view_data: ResMut<ViewData>,
) {

    if mouse.just_pressed(MouseButton::Left) {
        for (_entity, _node, mut transform) in query.iter_mut() {
            transform.translation.z = 60.0;
            view_data.top_z += 1.0;
        }
    }

    for _ev in ev_mouse_drag.iter() {
        if mouse.pressed(MouseButton::Left){
            for (_entity, _node, mut transform) in query.iter_mut() {
                    transform.translation.x += cursor.curr_position.x - cursor.prev_position.x;
                    transform.translation.y += cursor.curr_position.y - cursor.prev_position.y;     
            }
        }
        break
    }
}

#[derive(Event)]
struct MoveNodesEvent;

impl From<ListenerInput<Pointer<Drag>>> for MoveNodesEvent {
    fn from(_event: ListenerInput<Pointer<Drag>>) -> Self {
        MoveNodesEvent
    }
}

fn graph_pan(
    mut query: Query<&mut Transform, With<Camera2d>>,
    windows: Query<&Window>,
    mouse: Res<Input<MouseButton>>,
    view_settings: Res<ViewSettings>,
    _cursor: Res<InputData>,
    mut motion: EventReader<MouseMotion>,
) {
    let window = windows.single();
    let _cursor = window.cursor_position();

    if mouse.pressed(MouseButton::Middle) {
        for mut transform in query.iter_mut() {
                // transform.translation.x -= cursor.curr_position.x - cursor.prev_position.x;
                // transform.translation.y -= cursor.curr_position.y - cursor.prev_position.y; 
            for ev in motion.iter() {
                transform.translation.x -= ev.delta.x * view_settings.zoom;
                transform.translation.y += ev.delta.y * view_settings.zoom;
            }      
        }
        
    }
}

fn graph_zoom(
    mut query: Query<&mut OrthographicProjection, With<Camera2d>>,
    mut view_settings: ResMut<ViewSettings>,
    time: Res<Time>,
    mut events: EventReader<MouseWheel>,
) {
    let zoom_mult: f32 = 2.;

    for ev in events.iter() {
        match ev.unit {
            MouseScrollUnit::Line => {
                for mut projection in query.iter_mut() {
                    let mut log_scale = projection.scale.ln();
                    log_scale -= ev.y * zoom_mult * time.delta_seconds();
                    projection.scale = log_scale.exp();
                    view_settings.zoom = projection.scale;
        
                    println!("Current zoom scale: {}", projection.scale);
            }},
            MouseScrollUnit::Pixel => (),
        }
    }
}