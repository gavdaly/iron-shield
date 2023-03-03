use crate::index::generate_index;
use axum::{routing::get, Router};
use tower_http::services::ServeDir;

pub async fn run() {
    let app = Router::new()
        .route("/", get(generate_index))
        .nest_service("/static", ServeDir::new("static"));

    let address = &"0.0.0.0:3000".parse().expect("Error parsing address");

    axum::Server::bind(address)
        .serve(app.into_make_service())
        .await
        .expect("Run Server");

    println!("Site launched on: http://0.0.0.0:3000");
}
