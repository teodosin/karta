use agdb::{DbKeyValue, DbValue};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Attribute {
    pub name: String,
    pub value: AttrValue,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AttrValue {
    Float(f32),
    String(String),
    UInt(u32),
}

impl Into<DbValue> for AttrValue {
    fn into(self) -> DbValue {
        match self {
            AttrValue::Float(f) => DbValue::F64(f.into()),
            AttrValue::String(s) => DbValue::String(s),
            AttrValue::UInt(u) => DbValue::U64(u.into()),
        }
    }
}


impl Attribute {
    pub fn new_float(name: String, value: f32) -> Self {
        Self { name, value: AttrValue::Float(value) }
    }

    pub fn new_string(name: String, value: String) -> Self {
        Self { name, value: AttrValue::String(value) }
    }

    pub fn new_uint(name: String, value: u32) -> Self {
        Self { name, value: AttrValue::UInt(value) }
    }

    pub fn new_contains() -> Self {
        Self {
            name: "contains".to_string(),
            value: AttrValue::Float(0.0),
        }
    }
}
    
    impl From<serde_json::Value> for AttrValue {
        fn from(value: serde_json::Value) -> Self {
            match value {
                serde_json::Value::Number(n) => {
                    if let Some(f) = n.as_f64() {
                        AttrValue::Float(f as f32)
                    } else if let Some(u) = n.as_u64() {
                        AttrValue::UInt(u as u32)
                    } else {
                        // Fallback for other number types, maybe default to 0 or handle error
                        AttrValue::Float(0.0)
                    }
                }
                serde_json::Value::String(s) => AttrValue::String(s),
                // Add other cases as needed, e.g., Bool -> UInt
                _ => AttrValue::String("Unsupported value type".to_string()),
            }
        }
    }



impl Into<Vec<DbKeyValue>> for Attribute {
    fn into(self) -> Vec<DbKeyValue> {
        vec![
            match self.value {
                AttrValue::Float(f)     => DbKeyValue::from((self.name, f)),
                AttrValue::String(s)    => DbKeyValue::from((self.name, s)),
                AttrValue::UInt(u)      => DbKeyValue::from((self.name, u)),
            },
        ]
    }
}

impl Into<DbKeyValue> for Attribute {
    fn into(self) -> DbKeyValue {
        match self.value {
            AttrValue::Float(f)     => DbKeyValue::from((self.name, f)),
            AttrValue::String(s) => DbKeyValue::from((self.name, s)),
            AttrValue::UInt(u)      => DbKeyValue::from((self.name, u)),
        }
    }
}

impl Into<DbKeyValue> for &Attribute {
    fn into(self) -> DbKeyValue {
        match &self.value {
            AttrValue::Float(f)     => DbKeyValue::from((self.name.clone(), *f)),
            AttrValue::String(s) => DbKeyValue::from((self.name.clone(), s.clone())),
            AttrValue::UInt(u)      => DbKeyValue::from((self.name.clone(), *u)),
        }
    }
}

impl Into<Attribute> for DbKeyValue {
    fn into(self) -> Attribute {
        Attribute {
            name: self.key.to_string(),
            value: match self.value {
                DbValue::F64(f)     => AttrValue::Float(f.to_f64() as f32),
                DbValue::String(s) => AttrValue::String(s),
                DbValue::U64(u)       => AttrValue::UInt(u as u32),
                _ => panic!("Invalid attribute value"),
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SourceOrTarget {
    Source,
    Target,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RelativePosition {
    origin: SourceOrTarget,
    position: Vec<f64>
}

impl Into<Vec<DbKeyValue>> for RelativePosition {
    fn into(self) -> Vec<DbKeyValue> {
        let key = match self.origin {
            SourceOrTarget::Source => "source_position",
            SourceOrTarget::Target => "target_position",
        };
        vec![DbKeyValue::from((key.to_string(), self.position))]
    }
}

impl Into<DbKeyValue> for RelativePosition {
    fn into(self) -> DbKeyValue {
        let key = match self.origin {
            SourceOrTarget::Source => "source_position",
            SourceOrTarget::Target => "target_position",
        };
        DbKeyValue::from((key.to_string(), self.position))
    }
}

impl Into<DbKeyValue> for &RelativePosition {
    fn into(self) -> DbKeyValue {
        let key = match self.origin {
            SourceOrTarget::Source => "source_position",
            SourceOrTarget::Target => "target_position",
        };
        DbKeyValue::from((key.to_string(), self.position.clone()))
    }
}

impl From<DbKeyValue> for RelativePosition {
    fn from(kv: DbKeyValue) -> Self {
        let origin = if kv.key == "source_position".into() {
            SourceOrTarget::Source
        } else {
            SourceOrTarget::Target
        };
        let position = kv.value.vec_f64().unwrap();
        let position = position.into_iter().map(|f| {
            f.to_f64()
        }).collect();
        RelativePosition {
            origin,
            position,
        }
    }
}

impl TryFrom<&DbKeyValue> for Attribute {
    type Error = String;

    fn try_from(value: &DbKeyValue) -> Result<Self, Self::Error> {
        let name = value.key.to_string();
        let value = match &value.value {
            DbValue::F64(f) => AttrValue::Float(f.to_f64() as f32),
            DbValue::U64(u) => AttrValue::UInt(*u as u32),
            DbValue::String(s) => AttrValue::String(s.clone()),
            _ => return Err(format!("Unsupported DbValue type for attribute: {:?}", value.value)),
        };

        Ok(Attribute { name, value })
    }
}

/// A list of reserved node attribute names that cannot be set by the user directly.
/// 
/// NOTE:
/// This list was created before contexts were decided to exist in their own files instead of
/// that data being stored on edges. So many of these reservations aren't really needed anymore. 
pub const RESERVED_NODE_ATTRS: [&str; 13] = [
    "uuid",
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
    "source_transition", // Path to an animation file for when the edge is traversed in play mode. 
    "target_transition", // Path to an animation file for when the edge is traversed in play mode.

    "source_preload", // Preload settings for source node when in the target's context & play mode
    "target_preload", // Preload settings for the target node when in source node's context & play mode

    "source_output", // ID of an output socket in source node. Must be validated.
    "target_input", // ID of an input socket in target node. Must be validated. 

    // The following attributes are all Vecs of 2 f32s. 
    "source_position", // Relative position of source node to the target node
    "target_position", // Relative position of the target node to source node
    "source_scale", // Relative scale of source node to the target node
    "target_scale", // Relative scale of the target node to source node
    "source_rotation", // Relative rotation of source node to the target node
    "target_rotation", // Relative rotation of the target node to source node

    // The following attributes are all Vecs of 4 f32s. Or single hex values?
    "source_color", // Color of the source node when in the target's context
    "target_color", // Color of the target node when in the source node's context

    // The state pins of the node. 
    "source_pins", // The state pins of the source node when in the target's context
    "target_pins", // The state pins of the target node when in the source node's context

    // Bezier control points for the edge. 2 f32s for each point, arbitrary number of points.
    // If empty, the edge is a straight line.
    "source_bezier_control", // Control points for the bezier curve between the two nodes. 
    "target_bezier_control", // Control points for the bezier curve between the two nodes.

    // Karta needs a way to track whether a string of edges belongs to the same "sequence", indicating
    // that there is a preferred order to them. Use cases are for compiling 
    // "sequence"
];
