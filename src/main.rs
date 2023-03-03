use crate::index::generate_index;
use axum::{routing::get, Router};
mod config;
mod index;
mod sites;

#[tokio::main]
async fn main() -> Result<(), ()> {
    let app = Router::new().route("/", get(generate_index));

    let address = &"0.0.0.0:3000".parse().expect("Error parsing address");

    axum::Server::bind(address)
        .serve(app.into_make_service())
        .await
        .expect("Run Server");

    println!("Site launched on: http://0.0.0.0:3000");

    Ok(())
}
