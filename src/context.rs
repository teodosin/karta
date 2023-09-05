//context

#[derive(Resource, Debug)]
struct PathsToEntitiesIndex(
    HashMap<String, Entity>,
);

#[derive(Resource, Debug)]
struct KartaVault{
    vault_folder_name: String,
    root: String,
}

impl KartaVault {
    fn new() -> Self {
        KartaVault {
            vault_folder_name: "kartaVault".to_string(),
            root: "home".to_string(),
        }
    }

    fn get_root_path(&self) -> String {
        format!("{}/{}", self.root, self.vault_folder_name)
    }
}

// Spawn and despawn functions

// Spawn the context of a path
// Take in the path of a node
// Take in an optional filter 
// Check if the path has a corresponding context file
// If it does, spawn the nodes and edges in the context file
// If the path is a folder, spawn the contents of the folder as nodes and edges
// If the path is a file, spawn the file as a node
// Check the nodes against the filter, don't spawn if it doesn't match

// Do the reverse of the above for the despawn function


// Collapse and expand functions

// Similar to the spawn functions, but manages aliases also 
// So that when a node group is collapsed, it is replaced by its alias edge
// The edge that pointed to that node now points to the alias edge

// If a node group is expanded, the alias edge is replaced by the node group
// and their relevant edges.
// If an individual node is expanded and its file format is supported,
// its contents and their relevant edges are spawned around it (or in it)

