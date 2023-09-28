// Camera and cursor information for the graph

use bevy::{prelude::*, input::mouse::{MouseWheel, MouseScrollUnit, MouseMotion}};
use bevy_mod_picking::prelude::RaycastPickCamera;

pub struct GraphCamPlugin;

impl Plugin for GraphCamPlugin {
    fn build(&self, app: &mut App){
        app
            .insert_resource(ViewSettings::default())
            .insert_resource(ViewData::default())
            .insert_resource(InputData::default())

            .add_systems(Startup, cam_setup)

            .add_systems(PreUpdate, update_cursor_info)

            .add_systems(Update, graph_zoom)
            .add_systems(Update, graph_pan)

        ;
    }
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
    pub top_z: f32,
}

impl Default for ViewData {
    fn default() -> Self {
        ViewData { top_z: 0.0 }
    }
}


#[derive(Resource, Debug)]
pub struct InputData {
    pub latest_click_entity: Option<String>,
    pub latest_press_entity: Option<String>,
    pub latest_hover_entity: Option<String>,
    pub drag_position: Vec2,
    pub prev_position: Vec2,
    pub curr_position: Vec2,
}

impl Default for InputData {
    fn default() -> Self {
        InputData {
            latest_click_entity: None,
            latest_press_entity: None,
            latest_hover_entity: None,
            drag_position: Vec2::ZERO,
            prev_position: Vec2::ZERO,
            curr_position: Vec2::ZERO,
        }
    }
}

pub fn left_click_just_released(
    mouse: Res<Input<MouseButton>>,
) -> bool {
    mouse.just_released(MouseButton::Left)
}

fn cam_setup(
    mut commands: Commands,
){
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

    if mouse.just_released(MouseButton::Left) {
        cursor_history.latest_click_entity = None;
        cursor_history.latest_press_entity = None;
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

