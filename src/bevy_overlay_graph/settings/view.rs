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

#[derive(Resource, Debug)]
pub struct NodeInterpolationSettings {
    active: bool,
}

impl Default for NodeInterpolationSettings {
    fn default() -> Self {
        NodeInterpolationSettings {
            active: true,
        }
    }
}