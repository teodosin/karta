use agdb::DbKeyValue;

#[derive(Clone, Debug)]
pub struct Attribute {
    pub name: String,
    pub value: f32,
}

impl Attribute {
    pub fn new_contains() -> Self {
        Self {
            name: "contains".to_string(),
            value: 0.0,
        }
    }
}

impl Into<Vec<DbKeyValue>> for Attribute {
    fn into(self) -> Vec<DbKeyValue> {
        vec![
            DbKeyValue::from((self.name, self.value)),
        ]
    }
}

impl Into<DbKeyValue> for Attribute {
    fn into(self) -> DbKeyValue {
        DbKeyValue::from((self.name, self.value))
    }
}

impl Into<DbKeyValue> for &Attribute {
    fn into(self) -> DbKeyValue {
        DbKeyValue::from((self.name.clone(), self.value))
    }
}

impl Into<Attribute> for DbKeyValue {
    fn into(self) -> Attribute {
        Attribute {
            name: self.key.to_string(),
            value: self.value.to_f64().unwrap().to_f64() as f32,
        }
    }
}

/// A list of reserved node attribute names that cannot be set by the user directly.
pub const RESERVED_NODE_ATTRS: [&str; 12] = [
    "path", // The full path of the node, name included. Implemented as an alias, but still reserved.
    "name", // The name of the node, without the path. Maybe allows for different characters?

    "ntype", // The type of the node
    "nphys", // The physicality of the node

    "created_time", // The time when the node was created.
    "modified_time", // The time when the node was last modified.

    "preview", // Connects a file to a preview file, or stores it in this attribute in base64 for example. 

    "scale", // The absolute scaling of the node, in case it is needed. Vec of 2 f32s
    "rotation", // The absolute rotation of the node, in case it is needed. 
    "color", // The absolute color of the node. Vec of 4 f32s
    "pins", // The absolute state pins of the node. 

    // Reserved names with an underscore at the end are prefixes. 
    // All attributes with a name that starts with one of these prefixes are reserved.
    "param_", // Reserved prefix for any attribute that is a parameter, for operators for example. 
];

/// A list of reserved edge attribute names that cannot be set by the user directly.
/// Note that they are optional, so default behavior is when they are not set.
pub const RESERVED_EDGE_ATTRS: [&str; 22] = [
    "contains", // Physical parent_child relationship

    "text", // Text that is displayed on the edge, additional description

    "created_time", // The time the edge was created.
    "modified_time", // The time the edge was last modified.

    // Alternatively, transition animations could be stored in their own nodes. Might make it more 
    // explicit and ergonomic to edit them, and more easily support multiple animations. 
    // Or just have a vector of paths in the attributes below. 
    "from_transition", // Path to an animation file for when the edge is traversed in play mode. 
    "to_transition", // Path to an animation file for when the edge is traversed in play mode.

    "from_preload", // Preload settings for source node when in the target's context & play mode
    "to_preload", // Preload settings for the target node when in source node's context & play mode

    "from_output", // ID of an output socket in source node. Must be validated.
    "to_input", // ID of an input socket in target node. Must be validated. 

    // The following attributes are all Vecs of 2 f32s. 
    "from_position", // Relative position of source node to the target node
    "to_position", // Relative position of the target node to source node
    "from_scale", // Relative scale of source node to the target node
    "to_scale", // Relative scale of the target node to source node
    "from_rotation", // Relative rotation of source node to the target node
    "to_rotation", // Relative rotation of the target node to source node

    // The following attributes are all Vecs of 4 f32s. Or single hex values?
    "from_color", // Color of the source node when in the target's context
    "to_color", // Color of the target node when in the source node's context

    // The state pins of the node. 
    "from_pins", // The state pins of the source node when in the target's context
    "to_pins", // The state pins of the target node when in the source node's context

    // Bezier control points for the edge. 2 f32s for each point, arbitrary number of points.
    // If empty, the edge is a straight line.
    "from_bezier_control", // Control points for the bezier curve between the two nodes. 
    "to_bezier_control", // Control points for the bezier curve between the two nodes.

    // Karta needs a way to track whether a string of edges belongs to the same "sequence", indicating
    // that there is a preferred order to them. Use cases are for compiling 
    // "sequence"
];
