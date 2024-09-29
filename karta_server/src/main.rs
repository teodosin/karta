use fs_graph::prelude::run_server;

#[tokio::main]
async fn main() {
    run_server().await;
}
