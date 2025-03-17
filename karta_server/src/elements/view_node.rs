use uuid::Uuid;



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ViewNode {
    uuid: Uuid,

    position: (f32, f32),
    scale: (f32, f32),
    rotation: f32,
    rounded: f32,
}

impl ViewNode {
    pub fn new(uuid: Uuid) -> Self {
        Self {
            uuid,
            position: (0.0, 0.0),
            scale: (1.0, 1.0),
            rotation: 0.0,
            rounded: 0.0,
        }
    }

    pub fn scaled(&self, scale: (f32, f32)) -> Self {
        Self {
            uuid: self.uuid,
            position: self.position,
            scale,
            rotation: self.rotation,
            rounded: self.rounded,
        }
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }
}