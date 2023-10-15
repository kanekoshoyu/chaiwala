use crate::event;
use axum::body::Body;
use axum::extract::{Query, TypedHeader};
use axum::http::Response;
use axum::Extension;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

#[derive(Debug, Deserialize, Serialize)]
pub struct SetStatusParam {
    status: event::RuntimeStatus,
}

/// HTTP handler, returns plain text
pub async fn plain_hello_world(
    user_agent: Option<TypedHeader<headers::UserAgent>>,
) -> Response<Body> {
    log::info!("Connected: {}", user_agent.unwrap().as_str());
    Response::new(Body::from("Hello, World!".to_string()))
}

/// Set runtime status by query
pub async fn handler_set_status(
    Query(param): Query<SetStatusParam>,
    tx: Extension<Arc<Mutex<broadcast::Sender<event::RuntimeStatus>>>>,
) -> Response<Body> {
    let msg = format!(
        "Received REST command to set runtime status as [{:?}]",
        param.status
    );
    // Add your code to restart your service here.
    let tx = tx.lock().await;
    if tx.send(param.status).is_err() {
        log::warn!("problem publishing runtime status, check receiver side")
    }
    log::info!("{msg}");
    Response::new(Body::from(msg))
}
