use axum::{Extension, Router, Server};
use chaiwala::handler as chai_handler;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

/// broadcast numbers
async fn broadcast_numbers(tx: broadcast::Sender<i32>) {
    let mut i = 0;
    loop {
        tx.send(i).unwrap();
        // Simulate some delay before sending the next number
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        i += 1;
        if 10 == i {
            i = 0;
        }
    }
}

#[tokio::main]
async fn main() {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "example_websockets=debug,tower_http=debug")
    }
    chaiwala::logger::log_init();

    // Setup broadcast
    let (tx, rx) = broadcast::channel::<i32>(100);

    // Spawn a global number broadcast
    tokio::spawn(broadcast_numbers(tx));

    // router
    let app = Router::new()
        // HTTP
        .route(
            "/",
            axum::routing::get(chai_handler::http::plain_hello_world),
        )
        // Websocket
        .route(
            "/broadcast",
            axum::routing::get(chai_handler::ws::handler_broadcast),
        )
        .route(
            "/pingpong",
            axum::routing::get(chai_handler::ws::handler_ping_pong),
        )
        // Adds extension for broadcast receiver
        .layer(Extension(Arc::new(Mutex::new(rx))));

    // address
    let addr = SocketAddr::from(([0, 0, 0, 0], 1080));
    log::info!("listening on {}", addr);
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
