#![allow(warnings)]

mod utils;
use utils::*;
use std::path::PathBuf;

#[test]
fn new_node_has_type() {
    let func_name = "new_node_has_type";
    let mut graph = setup_graph(func_name);

    todo!();

    cleanup_graph(func_name);
}