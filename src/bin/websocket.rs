use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        TypedHeader,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "example_websockets=debug,tower_http=debug")
    }
    chaiwala::logger::log_init();

    let app = Router::new()
        .route("/", get(handle_http))
        //绑定websocket路由
        .route("/ws", get(handle_websocket));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    log::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handle_http() -> &'static str {
    "Hello, World!"
}

async fn handle_websocket(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
) -> impl IntoResponse {
    if let Some(TypedHeader(user_agent)) = user_agent {
        log::info!("Connected: {}", user_agent.as_str());
    }

    ws.on_upgrade(handle_websocket_client)
}

async fn handle_websocket_client(mut socket: WebSocket) {
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
