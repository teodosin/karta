// The idea of attributes is abstract enough to apply to both nodes and edges.
// As it's just a component added to either and Edge or Node entity, it can be
// used for both. 

use std::collections::HashMap;

use bevy::{prelude::Component, reflect::Reflect};

#[derive(Component, Reflect)]
pub struct Attributes {
    pub attributes: HashMap<String, Option<f32>>,
}
