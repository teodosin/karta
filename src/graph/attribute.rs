// The idea of attributes is abstract enough to apply to both nodes and edges.
// As it's just a component added to either and Edge or Node entity, it can be
// used for both. 

use std::collections::HashMap;

use bevy::{prelude::Component, reflect::Reflect};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub value: Option<f32>,
}

#[derive(Component, Reflect)]
pub struct Attributes {
    pub attributes: HashMap<String, Option<f32>>,
}

impl Attributes {
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
        }
    }

    pub fn add_attribute(&mut self, name: String, value: Option<f32>) {
        self.attributes.insert(name, value);
    }
}
