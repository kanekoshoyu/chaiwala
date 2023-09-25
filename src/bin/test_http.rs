use axum::{routing::get, Router, Server};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route("/", get(|| async { "Chaiwala!" }));

    // Set the server address.
    let socket_address: SocketAddr = SocketAddr::from(([0, 0, 0, 0], 1080));

    // run it with hyper on localhost:3000
    let server = Server::bind(&socket_address);
    server.serve(app.into_make_service()).await.unwrap();
}
