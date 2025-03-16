use crate::elements::view_node::ViewNode;


pub struct ContextDb {
    name: String, 
    root_path: PathBuf,
}


struct Context {
    karta_version: String,
    focal: ViewNode,
    nodes: Vec<ViewNode>,
}