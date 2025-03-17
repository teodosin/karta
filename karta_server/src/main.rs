// use karta_server::prelude::run_server;

use karta_server::prelude::{load_or_create_vault, run_server};

#[tokio::main]
async fn main() {
    let vault_path = load_or_create_vault();
    let vault_path = match vault_path {
        Ok(path) => path,
        Err(_) => {
            println!("No vault selected. Exiting...");
            std::process::exit(1);
        }
    };
    run_server(vault_path).await;
}
