
use axum::{extract::Path, Extension, Json};

use crate::prelude::NodePath;

use super::AppState;









pub async fn root() -> &'static str {
    "Welcome to Karta Server"
}

// pub async fn get_all_aliases(Extension(state): Extension<AppState>) -> Json<Vec<String>> {
//     let graph = &state.graph_commands.read().unwrap();
//     let aliases = graph.get_all_aliases();
//     Json(aliases)
// }

// pub async fn get_root_node(Extension(state): Extension<AppState>) -> Json<Result<Node, String>> {
//     let graph = &state.graph_commands.read().unwrap();

//     let root_path = NodePath::root();
//     let result = graph
//         .open_node(&root_path)
//         .map_err(|e| e.to_string());
//     Json(result)
// }

// pub async fn get_node(
//     Extension(state): Extension<AppState>,
//     Path(id): Path<String>,
// ) -> Json<Result<Node, String>> {
//     let graph = &state.graph_commands.read().unwrap();

//     // println!("Requested node with id: {}", id);
//     let node_path = NodePath::from_alias(&id);
//     // println!("Resulting node_path: {:#?}", node_path);
//     // println!("Resulting alias: {}", node_path.alias());
//     let result = graph
//         .open_node(&node_path)
//         .map_err(|e| e.to_string());
//     Json(result)
// }

// pub async fn get_node_context(
//     Extension(state): Extension<AppState>,
//     Path(id): Path<String>,
// ) -> Json<Vec<(Node, Edge)>> {
//     let graph = &state.graph_commands.read().unwrap();
//     let node_path = NodePath::from_alias(&id);
//     let result = graph.open_node_connections(&node_path);
//     Json(result)
// }