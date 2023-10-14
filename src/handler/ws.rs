use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::TypedHeader;
use axum::Extension;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

/// WebSocket handler, returns response from callback
pub async fn handler_broadcast(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    rx: Extension<Arc<Mutex<broadcast::Receiver<i32>>>>,
) -> impl axum::response::IntoResponse {
    log::info!("Connected: {}", user_agent.unwrap().as_str());
    ws.on_upgrade(move |socket: WebSocket| publish_index(socket, rx.0))
}

/// Websocket Callback that sends received data from broadcast
async fn publish_index(mut ws: WebSocket, rx: Arc<Mutex<broadcast::Receiver<i32>>>) {
    while let Ok(number) = rx.lock().await.recv().await {
        ws.send(Message::Text(format!("{number}"))).await.unwrap();
    }
    ws.close().await.unwrap();
}

/// WebSocket handler, returns response from callback
pub async fn handler_ping_pong(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
) -> impl axum::response::IntoResponse {
    if let Some(TypedHeader(user_agent)) = user_agent {
        log::info!("Connected: {}", user_agent.as_str());
    }

    ws.on_upgrade(pub_received)
}

/// Websocket Callback that sends received data
async fn pub_received(mut socket: WebSocket) {
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
