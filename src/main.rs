mod config;
mod index;
mod server;
mod sites;

#[tokio::main]
async fn main() {
    server::run().await;
}
