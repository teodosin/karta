// Module that handles and tracks assets
// For now its function will be to keep track of the loading of images and svgs
// and send update events to the ui when they are loaded

use bevy::{ecs::{system::{Resource, Commands, ResMut, Query}, event::EventReader, entity::Entity, query::Without}, asset::{Handle, AssetEvent, Assets}, render::{texture::Image, mesh::{shape, Mesh}, color::Color}, sprite::{Sprite, MaterialMesh2dBundle, ColorMaterial, Mesh2dHandle}, transform::components::Transform, math::{Vec2, Vec3, Rect}, hierarchy::{Parent, Children}, prelude::default};

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

    mut nodes: Query<(Entity, Option<&Children>, &Handle<Image>, &mut Sprite, &mut Transform), (Without<NodeOutline>, Without<NodeLabel>)>,
    mut labels: Query<(&NodeLabel, &mut Transform), Without<NodeOutline>>,
    mut outlines: Query<(&NodeOutline, &mut Transform), Without<NodeLabel>>,

    // Because sprite picking isn't working in Karta for some reason, we will spawn a mesh to be the pick target
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
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

                for (_node, children, img, mut sprite, mut form) in nodes.iter_mut() {
                    if img == &bevy::prelude::Handle::Weak(id.clone()) {
                        println!("Image loaded: {:?}", id);
                        // Should be true when loaded with deps
                        if img.is_strong(){
                            
                            let img = image_assets.get(img.clone()).unwrap();
                            let res = Vec2::new(img.texture_descriptor.size.width as f32, img.texture_descriptor.size.height as f32);
                            
                            // sprite.rect = Some(Rect {
                            //     min: Vec2::new(0.0, 0.0),
                            //     max: Vec2::new(res.x, res.y),
                            // });
                            // commands.entity(node).insert(
                            //         Mesh2dHandle { 
                            //             0: meshes.add(shape::Quad {
                            //                 size: Vec2::new(res.x, res.y),
                            //                 flip: false,
                            //             }.into()).into(),
                            // });
                            
                            let larger = res.x.max(res.y);
                            let base_scale = 100.0;
                            
                            let scale = Vec3::new(base_scale / larger, base_scale / larger, 0.0);
                            let reverse_scale = Vec3::new(larger / base_scale, larger / base_scale, 0.0);
                            form.scale = scale;
                            println!("Resized image to {:?}", scale);


                            //Reverse the scale for the children of the node
                            match children {
                                Some(children) => {
                                    for child in children.iter() {
                                        match labels.get_mut(*child) {
                                            Ok(mut label) => {
                                                println!("Rescaling label");
                                                label.1.scale = reverse_scale;
                                            }
                                            Err(_) => {}
                                        }
                                        match outlines.get_mut(*child) {
                                            Ok(mut outline) => {
                                                println!("Rescaling outline");
                                                outline.1.scale = reverse_scale;
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
