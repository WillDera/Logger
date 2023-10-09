mod dto;
mod routes;

use env_logger;
use std::env;
use std::fs::create_dir;
use std::path::Path;

use routes::create_routes;

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "INFO");
    env_logger::init();

    // check if the log folder exists, else create it
    let log_folder = Path::new("log_folder/");
    if log_folder.exists() {
        log::info!("Log Folder Found");
    } else {
        log::info!("Creating Log Folder");
        create_dir("log_folder/").unwrap();
    }

    // build our application with a single route
    let app = create_routes();

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
