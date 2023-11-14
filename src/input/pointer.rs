use std::path::PathBuf;

use bevy::{prelude::{Vec2, ResMut, Input, Res, MouseButton, Query, Camera, GlobalTransform, With, Camera2d, Resource}, window::Window};



#[derive(Resource, Debug)]
pub struct InputData {
    pub latest_click_entity: Option<PathBuf>,
    pub latest_press_entity: Option<PathBuf>,
    pub latest_hover_entity: Option<PathBuf>,

    pub press_is_outline: bool,

    pub left_just_released: bool,

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

            press_is_outline: false,

            left_just_released: false,

            drag_position: Vec2::ZERO,
            prev_position: Vec2::ZERO,
            curr_position: Vec2::ZERO,
        }
    }
}

pub fn left_click_just_released(
    input: Res<InputData>,
) -> bool {
    input.left_just_released
}

pub fn update_cursor_info(
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
        //cursor_history.latest_press_entity = None;
        cursor_history.left_just_released = true;
    }
    else {
        cursor_history.left_just_released = false;
    }
}