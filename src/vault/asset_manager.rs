// Module that handles and tracks assets
// For now its function will be to keep track of the loading of images and svgs
// and send update events to the ui when they are loaded

use bevy::{ecs::{system::{Resource, Commands, ResMut, Query}, event::EventReader, entity::Entity, query::Without}, asset::{Handle, AssetEvent, Assets}, render::texture::Image, sprite::Sprite, transform::components::Transform, math::{Vec2, Vec3}, hierarchy::{Parent, Children}};

use crate::ui::nodes::{GraphViewNode, NodeLabel, NodeOutline};

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
    mut commands: Commands,
    mut image_tracker: ResMut<ImageLoadTracker>,
    mut image_events: EventReader<AssetEvent<Image>>,
    image_assets: ResMut<Assets<Image>>,

    mut nodes: Query<(Entity, &Children, &Handle<Image>, &mut Transform), (Without<NodeOutline>, Without<NodeLabel>)>,
    mut labels: Query<(&NodeLabel, &mut Transform), Without<NodeOutline>>,
    mut outlines: Query<(&NodeOutline, &mut Transform), Without<NodeLabel>>,
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
            AssetEvent::LoadedWithDependencies { id } => {
                image_tracker.remove_image(bevy::prelude::Handle::Weak(id.clone()));

                for (_node, children, sprite, mut form) in nodes.iter_mut() {
                    if sprite == &bevy::prelude::Handle::Weak(id.clone()) {
                        println!("Image loaded: {:?}", id);
                        // Should be true when loaded with deps
                        if sprite.is_strong(){
                            let img = image_assets.get(sprite.clone()).unwrap();
                            let res = Vec2::new(img.texture_descriptor.size.width as f32, img.texture_descriptor.size.height as f32);
                            let larger = res.x.max(res.y);
                            let base_scale = 100.0;

                            let scale = Vec3::new(base_scale / larger, base_scale / larger, 0.0);
                            let reverse_scale = Vec3::new(larger / base_scale, larger / base_scale, 0.0);
                            form.scale = scale;

                            //Reverse the scale for the children of the node
                            for child in children.iter() {
                                match labels.get_mut(*child) {
                                    Ok(_) => {
                                        println!("Rescaling label");
                                        labels.get_mut(*child).unwrap().1.scale = reverse_scale;
                                    }
                                    Err(_) => {}
                                }
                                match outlines.get_mut(*child) {
                                    Ok(_) => {
                                        println!("Rescaling outline");
                                        outlines.get_mut(*child).unwrap().1.scale = reverse_scale;
                                    }
                                    Err(_) => {}
                                }
                                
                            }

                        }
                    }
                }
            }
        }
    }
}
