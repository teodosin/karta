use bevy::prelude::Resource;

#[derive(Resource, Debug)]
pub struct EdgeDefaults {
    length: f32,
}

impl Default for EdgeDefaults {
    fn default() -> Self {
        EdgeDefaults {
            length: 100.0,
        }
    }
}