// Camera and cursor information for the graph

use bevy::{prelude::*, input::mouse::{MouseWheel, MouseScrollUnit, MouseMotion}, render::view::RenderLayers};

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
    top_z: f32,
    bottom_z: f32,
    increment: f32,
}

impl Default for ViewData {
    fn default() -> Self {
        ViewData { 
            top_z: 0.0,
            bottom_z: 0.0,
            increment: 0.001,
        }
    }
}

impl ViewData {
    pub fn get_z_for_edge(&mut self) -> f32 {
        self.bottom_z = self.bottom_z - self.increment;
        self.bottom_z
    }

    pub fn get_z_for_node(&mut self) -> f32 {
        self.top_z = self.top_z + self.increment;
        self.top_z
    }
}

#[derive(Component)]
pub struct GraphCamera;

/// Set up the camera for the graph. 
/// Bevy doesn't seem to currently support drawing meshes or arbitrary shapes
/// in the UI, so the graph exists currently in world space. 
/// 
/// To make the graph not interfere with the rest of the world, the graph elements
/// will be set to render in the 32nd (last) render layer.
fn cam_setup(
    mut commands: Commands,
){
    use bevy::core_pipeline::clear_color::ClearColorConfig;
    commands.spawn((
        GraphCamera,
        Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::None,
                ..default()
            },
            projection: OrthographicProjection {
                far: 1000000.,
                near: -1000000.,
                ..Default::default()
            },
            camera: Camera {
                order: 1,
                ..default()
            },
            ..default()
        },
        // The graph exists in world space, and we don't want it to interfere with the rest of the world. 
        RenderLayers::from_layers(&[31]),
        bevy_mod_picking::backends::raycast::RaycastPickable,
    ));
}



fn graph_pan(
    mut query: Query<&mut Transform, With<Camera2d>>,
    mouse: Res<Input<MouseButton>>,
    view_settings: Res<ViewSettings>,
    _cursor: Res<InputData>,
    mut motion: EventReader<MouseMotion>,
) {

    for ev in motion.read() {
        if mouse.pressed(MouseButton::Middle) {
            for mut transform in query.iter_mut() {
                // transform.translation.x -= cursor.curr_position.x - cursor.prev_position.x;
                // transform.translation.y -= cursor.curr_position.y - cursor.prev_position.y; 
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

    for ev in events.read() {
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

