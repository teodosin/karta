// The idea of attributes is abstract enough to apply to both nodes and edges.
// As it's just a component added to either and Edge or Node entity, it can be
// used for both. 

#[derive(Component, Reflect)]
pub struct Attributes {
    pub attributes: HashMap<Attribute>,
}

pub struct Attribute {
    // Maybe redundant, if the hashmap key is the name
    pub name: String, 
    // An attribute without a value is just a tag
    pub value: Option<f32>,
}