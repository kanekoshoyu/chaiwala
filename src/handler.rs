use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::TypedHeader;
use axum::Extension;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

/// HTTP handler, returns plain text
pub async fn handle_http(user_agent: Option<TypedHeader<headers::UserAgent>>) -> &'static str {
    log::info!("Connected: {}", user_agent.unwrap().as_str());
    "Hello, World!"
}

/// WebSocket handler, returns response from callback
pub async fn handle_ws_broadcast(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    rx: Extension<Arc<Mutex<broadcast::Receiver<i32>>>>,
) -> impl axum::response::IntoResponse {
    // callback upon reception
    log::info!("Connected: {}", user_agent.unwrap().as_str());

    ws.on_upgrade(move |socket: WebSocket| ws_upgrade_callback(socket, rx.0))
}

/// Websocket Callback that sends received data from broadcast
async fn ws_upgrade_callback(mut ws: WebSocket, rx: Arc<Mutex<broadcast::Receiver<i32>>>) {
    // TODO spawn both the broadcast loop and the receiver loop for real-time control
    // while websocket is on connection
    while let Ok(number) = rx.lock().await.recv().await {
        ws.send(Message::Text(format!("{number}"))).await.unwrap();
    }
    // sends Message::Close()
    ws.close().await.unwrap();
}

/// WebSocket handler, returns response from callback
pub async fn handle_ws_pingpong(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
) -> impl axum::response::IntoResponse {
    if let Some(TypedHeader(user_agent)) = user_agent {
        log::info!("Connected: {}", user_agent.as_str());
    }

    ws.on_upgrade(ws_callback_pingpong)
}

/// Websocket Callback that sends received data
async fn ws_callback_pingpong(mut socket: WebSocket) {
    loop {
        let res = socket.recv().await;
        if res.is_none() {
            break;
        }
        let res = res.unwrap();
        if let Err(e) = res {
            log::warn!("Failed receiving message {e}");
            break;
        }
        let msg = res.unwrap();
        log::info!("RX: {:?}", msg);
        if let Message::Close(_) = msg {
            log::warn!("Close message received");
            break;
        }
        if let Message::Text(text) = msg {
            let res = socket.send(Message::Text(text.clone())).await;
            if res.is_err() {
                log::warn!("Failed sending message, disconnecting client");
                return;
            }
            log::info!("TX: {:?}", text);
        }
    }
    log::warn!("Escaping handler");
}
