// Camera and cursor information for the graph

use bevy::{prelude::*, input::mouse::{MouseWheel, MouseScrollUnit, MouseMotion}};
use bevy_mod_picking::prelude::RaycastPickCamera;

use crate::input::pointer::InputData;

pub struct GraphCamPlugin;

impl Plugin for GraphCamPlugin {
    fn build(&self, app: &mut App){
        app
            .insert_resource(ViewSettings::default())
            .insert_resource(ViewData::default())

            .add_systems(Startup, cam_setup)

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
    mut query: Query<(&mut OrthographicProjection, &mut Transform), With<Camera2d>>,
    input_data: Res<InputData>,
    mut view_settings: ResMut<ViewSettings>,
    _time: Res<Time>,
    mut events: EventReader<MouseWheel>,
) {
    let zoom_mult: f32 = 1.07;

    for ev in events.iter() {
        match ev.unit {
            MouseScrollUnit::Line => {
                for (mut projection, mut transform) in query.iter_mut() {

                    match ev.y {
                        y if y > 0.0 => { // ZOOMING IN
                            projection.scale = projection.scale / zoom_mult;
                            view_settings.zoom = projection.scale;

                            // Zoom-in is centered on mouse position
                            let amount = zoom_mult - 1.0;
                            let adjusted_position = (input_data.curr_position - transform.translation.truncate()) * amount;
                            transform.translation.x += adjusted_position.x;
                            transform.translation.y += adjusted_position.y;
                        },
                        y if y < 0.0 => { // ZOOMING OUT
                            projection.scale = projection.scale * zoom_mult;
                            view_settings.zoom = projection.scale;
                        },
                        _ => (),
                    }
        
                    println!("Current zoom scale: {}", projection.scale);

            }},
            MouseScrollUnit::Pixel => (),
        }
    }
}

