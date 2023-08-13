use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::Extension;
use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

async fn handle_http() -> &'static str {
    "Hello, World!"
}

/// Spawn number generator and websocket callback
async fn handle_ws(
    ws: WebSocketUpgrade,
    rx: Extension<Arc<Mutex<broadcast::Receiver<i32>>>>,
) -> impl axum::response::IntoResponse {
    // callback upon reception
    ws.on_upgrade(move |socket: WebSocket| ws_upgrade_callback(socket, rx.0))
}

/// Send number to the broadcast
async fn generate_numbers(tx: broadcast::Sender<i32>) {
    for i in 0..10 {
        // Sending the number through the broadcast channel
        tx.send(i).unwrap();
        // Simulate some delay before sending the next number
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

// Websocket Callback
async fn ws_upgrade_callback(mut ws: WebSocket, rx: Arc<Mutex<broadcast::Receiver<i32>>>) {
    // while websocket is on connection
    while let Ok(number) = rx.lock().await.recv().await {
        ws.send(Message::Text(format!("{number}"))).await.unwrap();
    }
    // sends Message::Close()
    ws.close().await.unwrap();
}

#[tokio::main]
async fn main() {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "example_websockets=debug,tower_http=debug")
    }
    chaiwala::logger::log_init();

    // Setup broadcast
    let (tx, rx) = broadcast::channel::<i32>(100);

    let arc_rx = Arc::new(Mutex::new(rx));

    // Clone the sender to pass it to the broadcast loop
    tokio::spawn(generate_numbers(tx));

    let app = Router::new()
        // HTTP
        .route("/", axum::routing::get(handle_http))
        // Websocket
        .route("/ws", axum::routing::get(handle_ws))
        // Adds extension
        .layer(Extension(arc_rx));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    log::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
