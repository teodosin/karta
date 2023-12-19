// 

use std::{fs, fmt, path::PathBuf};

use bevy::asset::{AssetServer, Handle};
use bevy::prelude::{Plugin, App};
use bevy::ecs::world::World;
use bevy::render::texture::Image;
use bevy_svg::prelude::Svg;
use enum_iterator::Sequence;
use serde::{Serialize, Deserialize};

mod file_types;
mod filters;
mod forces;
mod operators;
mod panels;
mod query;

pub struct NodeTypesPlugin;

impl Plugin for NodeTypesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(forces::ForceNodesPlugin)
        ;
    }
}

// For now, all node types will be stored in a single enum
// This will be changed to a more flexible system later
#[derive(Clone, Copy, Debug, PartialEq, Sequence, Serialize, Deserialize)]
pub enum NodeTypes {
    Base,
    Folder, 
    FileBase,
    FileImage,
    FileSvg,
    FileText,
    Text,
}

impl fmt::Display for NodeTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NodeTypes::Base => write!(f, "Base"),
            NodeTypes::Folder => write!(f, "Folder"),
            NodeTypes::FileBase => write!(f, "Generic File"),
            NodeTypes::FileImage => write!(f, "Image"),
            NodeTypes::FileSvg => write!(f, "Svg"),
            NodeTypes::FileText => write!(f, "Text File"),
            NodeTypes::Text => write!(f, "Text Card"),
        }
    }

}

impl Default for NodeTypes {
    fn default() -> Self {
        NodeTypes::Base
    }
}

#[derive(Clone, Debug, PartialEq, Sequence)]
pub enum DataTypes {
    None,
    TypeSvg,
    TypeImage,
    TypeText,
}

pub fn type_to_data(
    ntype: NodeTypes,
) -> Option<Box<dyn NodeData>>  {
    let data = match ntype {
        NodeTypes::Base => None,
        NodeTypes::Folder => None,
        NodeTypes::FileBase => None,
        NodeTypes::FileImage => {
            let data: Box<dyn NodeData> = Box::new(TypeImage {
                image: None,
            });
            Some(data)
        },
        NodeTypes::FileSvg => {
            let data: Box<dyn NodeData> = Box::new(TypeSvg {
                svg: None,
            });
            Some(data)
        },
        NodeTypes::FileText => None,
        NodeTypes::Text => None,
    };
    data
}

impl fmt::Display for DataTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataTypes::None => write!(f, "No data"),
            DataTypes::TypeText => write!(f, "Text"),
            DataTypes::TypeImage => write!(f, "Image"),
            DataTypes::TypeSvg => write!(f, "Svg"),
        }
    }
}

/// A helper function to get the type of a physical node based on its path
pub fn get_type_from_file_path(
    path: &PathBuf, 
) -> Option<NodeTypes> {
    match fs::metadata(&path) {
        Ok(metadata) => {
            if metadata.is_dir() {
                Some(NodeTypes::Folder)
            } else {
                let ext = path.extension();
                match ext {
                    Some(ext) => {
                        println!("Matching extensions: {:?}", ext);
                        match ext.to_str() {
                            Some("svg") => return Some(NodeTypes::FileSvg),
                            Some("png") => return Some(NodeTypes::FileImage),
                            Some("jpg") => return Some(NodeTypes::FileImage),
                            Some("jpeg") => return Some(NodeTypes::FileImage),
                            Some("txt") => return Some(NodeTypes::FileText),
                            _ => return Some(NodeTypes::FileBase),
                        }
                    },
                    None => return Some(NodeTypes::FileBase),
                }
            }
        },
        Err(_) => {
            println!("Error: Parent path does not exist");
            None
        }
    }
}

pub fn get_type_from_context_path(
    path: &PathBuf, 
) -> Option<NodeTypes> {
    Some(NodeTypes::Base) 
}

pub trait NodeData: Send + Sync + 'static {
    // Path and node are stored in the GraphNode, right?
    // fn get_path(&self) -> String;
    // fn get_name(&self) -> String;
    fn get_data_type(&self) -> String;

    fn set_data(&mut self);

    fn get_data(&self, world: &World, path: &PathBuf) -> Box<dyn NodeData>;
}

pub struct TypeText {
    pub text: Option<String>,
}

impl NodeData for TypeText {
    fn get_data_type(&self) -> String {
        DataTypes::TypeText.to_string()
    }

    fn set_data(&mut self) {
        println!("Setting data");
    }

    fn get_data(&self, _world: &World, _path: &PathBuf) -> Box<dyn NodeData> {
        let data = TypeText {
            text: self.text.clone(),
        };
        Box::new(data)
    }
}
pub struct TypeSvg {
    pub svg: Option<Handle<Svg>>,
}

impl NodeData for TypeSvg {
    fn get_data_type(&self) -> String {
        DataTypes::TypeSvg.to_string()
    }

    fn set_data(&mut self) {
        println!("Setting data");
    }

    fn get_data(&self, world: &World, path: &PathBuf) -> Box<dyn NodeData> {
        match &self.svg {
            Some(svg) => {
                let data = TypeSvg {

                    svg: Some(svg.clone()),
                };
                println!("Found svg handle");
                Box::new(data)
            },
            None => {
                // Here we see that the file is not loaded, so we load it 
                let server = world.get_resource::<AssetServer>().unwrap();

                // Check that the file is an svg
                if path.extension().unwrap() != "svg" {
                    println!("Error: File is not an svg");
                    return Box::new(TypeSvg {
                        svg: None,
                    });
                }

                let svg_file: Handle<Svg> = server.load(path.clone());
                
                let data = TypeSvg {
                    svg: Some(svg_file),
                };
                Box::new(data)
            }
        }
    }
}

pub struct TypeImage {
    pub image: Option<Handle<Image>>,
}

impl NodeData for TypeImage {
    fn get_data_type(&self) -> String {
        DataTypes::TypeImage.to_string()
    }

    fn set_data(&mut self) {
        println!("Setting data");
    }

    fn get_data(&self, world: &World, path: &PathBuf) -> Box<dyn NodeData> {
        println!("Getting data for image: {:?}", path);
        match &self.image {
            Some(image) => {
                let data = TypeImage {
                    image: Some(image.clone()),
                };
                Box::new(data)
            },
            None => {
                // Here we see that the file is not loaded, so we load it 
                let server = world.get_resource::<AssetServer>().unwrap();
                let image = server.load(path.clone());
                let data = TypeImage {
                    image: Some(image),
                };
                Box::new(data)
            }
        }
    }
}
