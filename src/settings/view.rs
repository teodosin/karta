use bevy::prelude::Resource;

#[derive(Resource, Debug)]
pub struct EdgeDefaults {
    _length: f32,
}

impl Default for EdgeDefaults {
    fn default() -> Self {
        EdgeDefaults {
            _length: 100.0,
        }
    }
}