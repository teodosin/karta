use uuid::Uuid;




pub struct ViewNode {
    id: Uuid,

    position: (f32, f32),
    scale: (f32, f32),
    rotation: f32,
    rounded: f32,
}