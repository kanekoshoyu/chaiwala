use crate::event;
use crate::handler;
use axum::body::Body;
use axum::http::Response;
use axum::routing::get;
use axum::{Extension, Router, Server};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

pub async fn task_api_router(tx_status: broadcast::Sender<event::RuntimeStatus>) -> Result<(), failure::Error> {
    // mutex since handlers gets spawned
    let tx_cmd_ref = Arc::new(Mutex::new(tx_status));
    let app = Router::new()
        .route("/", get(handler::http::plain_hello_world))
        .route("/set", get(handler::http::handler_set_status))
        .route("/status", get(handler_status))
        .layer(Extension(tx_cmd_ref.clone()));

    // Set the server address.
    let localhost = [0, 0, 0, 0];
    let port = 1080;
    let socket_address: SocketAddr = SocketAddr::from((localhost, port));

    log::info!("Setup server at [{socket_address:?}]");
    // run server
    let server = Server::bind(&socket_address);
    Ok(server.serve(app.into_make_service()).await?)
}

async fn handler_status() -> Response<Body> {
    // Add your code to terminate your service here.
    let status: bool = false;
    let msg: String = format!("status: {status:?}");
    log::info!("{msg}");
    Response::new(Body::from(msg))
}
