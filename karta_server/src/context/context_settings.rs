use serde::{Deserialize, Serialize};



#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContextSettings {
    zoom_scale: f32,
    view_rel_pos_x: f32,
    view_rel_pos_y: f32,
}

impl Default for ContextSettings {
    fn default() -> Self {
        Self {
            zoom_scale: 1.0,
            view_rel_pos_x: 0.0,
            view_rel_pos_y: 0.0,
        }
    }
}