use axum::{routing::get, Router, Server};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route("/", get(|| async { "Chaiwala!" }));

    // Set the server address.
    let socker_address = SocketAddr::from(([127, 0, 0, 1], 3000));

    // run it with hyper on localhost:3000
    let server = Server::bind(&socker_address);
    server.serve(app.into_make_service()).await.unwrap();
}
