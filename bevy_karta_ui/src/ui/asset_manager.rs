// Module that handles and tracks assets
// For now its function will be to keep track of the loading of images and svgs
// and send update events to the ui when they are loaded


use bevy::{
    ecs::{system::{Resource, ResMut, Query}, event::EventReader, entity::Entity, query::Without}, 
    asset::{Handle, AssetEvent, Assets}, 
    render::texture::Image, sprite::{Sprite, Mesh2dHandle}, transform::components::Transform, math::{Vec2, Vec3}, 
    hierarchy::Children, 
};
use bevy_prototype_lyon::{shapes, entity::Path, geometry::GeometryBuilder, draw::Stroke};

use super::nodes::{ViewNodeShape, NodeOutline, NodeLabel};

#[derive(Resource)]
pub struct ImageLoadTracker {
    pub images: Vec<Handle<Image>>,
}

impl ImageLoadTracker {
    pub fn new() -> Self {
        ImageLoadTracker {
            images: Vec::new(),
        }
    }

    pub fn add_image(&mut self, image: Handle<Image>) {
        self.images.push(image);
    }

    pub fn remove_image(&mut self, image: Handle<Image>) {
        self.images.retain(|x| *x != image);
    }
}

pub fn on_image_load(    
    mut image_tracker: ResMut<ImageLoadTracker>,
    mut image_events: EventReader<AssetEvent<Image>>,
    image_assets: ResMut<Assets<Image>>,

    mut nodes: Query<(
        Entity, Option<&Children>, &Handle<Image>, &mut ViewNodeShape, &mut Sprite, &mut Transform), 
        (Without<NodeOutline>, Without<NodeLabel>, Without<Mesh2dHandle>)>,
    mut labels: Query<(&NodeLabel, &mut Transform), Without<NodeOutline>>,
    mut outlines: Query<(&NodeOutline, &mut Path, &Stroke), Without<NodeLabel>>,

) {

    
    for event in image_events.read() {
        match event {
            AssetEvent::Added { id: _ } => {} // We are adding the image to the tracker where it is loaded 
            AssetEvent::Removed { id } => {
                image_tracker.remove_image(bevy::prelude::Handle::Weak(id.clone()));
            }
            AssetEvent::Modified { id } => {
                image_tracker.remove_image(bevy::prelude::Handle::Weak(id.clone()));
                image_tracker.add_image(bevy::prelude::Handle::Weak(id.clone()));
            }
            AssetEvent::Unused { id } => {
                image_tracker.remove_image(bevy::prelude::Handle::Weak(id.clone()));
            }
            AssetEvent::LoadedWithDependencies { id } => {
                image_tracker.remove_image(bevy::prelude::Handle::Weak(id.clone()));
                
                for (node, children, img, mut vshp, mut sprite, form) in nodes.iter_mut() {
                    // if img.is_strong() {
                    if img == &bevy::prelude::Handle::Weak(id.clone()) {
                        // Should be true when loaded with deps
                        if img.is_strong(){
                            
                            let img = match image_assets.get(img){
                                Some(img) => img,
                                None => continue,
                            };
                            
                            let res = Vec2::new(img.texture_descriptor.size.width as f32, img.texture_descriptor.size.height as f32);

                            let larger = res.x.max(res.y);
                                        
                            let base_scale = 100.0;
                            
                            let scale = Vec2::new(base_scale / larger * res.x, base_scale / larger * res.y);

                            sprite.custom_size = Some(scale);
                            *vshp = ViewNodeShape::Rectangle(scale);

                            // Resize outline and move label


                            match children {
                                Some(children) => {
                                    for child in children.iter() {
                                        match labels.get_mut(*child) {
                                            Ok(mut label) => {
                                                let z = label.1.translation.z;
                                                let label_pos = Vec3::new(-scale.x / 2.0, scale.y / 2.0 + 10.0, z);

                                                label.1.translation = label_pos;
                                            }
                                            Err(_) => {}
                                        }
                                        match outlines.get_mut(*child) {
                                            Ok(mut outline) => {
                                                let outline_width = outline.2.options.line_width;
                                                let outline_shape = shapes::Rectangle {
                                                    extents: scale + Vec2::new(outline_width, outline_width),
                                                    origin: shapes::RectangleOrigin::Center,
                                                };
                                                let outline_path = GeometryBuilder::build_as(&outline_shape);

                                                *outline.1 = outline_path;

                                            }
                                            Err(_) => {}
                                        }
                                        
                                    }

                                }
                                None => {}
                            }
                        }
                    }
                }
            }
        }
    }
}
