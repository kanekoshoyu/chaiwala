use axum::body::Body;
use axum::http::Response;
use axum::routing::get;
use axum::{Extension, Router, Server};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::{broadcast, Mutex};
use tokio::time::sleep;

async fn runtime() -> Result<(), failure::Error> {
    println!("Bot is running");
    let duration = Duration::from_secs(5);
    loop {
        sleep(duration).await;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum HTTPCommand {
    Start,
    Stop,
}

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // server should always be available, whereas the arbitrage code should be controllable
    // build our application with a single route
    let (tx_command, _) = broadcast::channel::<HTTPCommand>(4);
    let (tx_restart, mut rx_restart) = broadcast::channel::<()>(1);
    let tx_cmd_ref = Arc::new(Mutex::new(tx_command));

    let app = Router::new()
        .route("/", get(|| async { "KuCoin Arbitrage is running" }))
        .route("/start", get(handler_start))
        .route("/status", get(handler_status))
        .route("/stop", get(handler_stop))
        .layer(Extension(tx_cmd_ref.clone()));

    // Set the server address.
    let localhost = [0, 0, 0, 0];
    let port = 1080;
    let socket_address: SocketAddr = SocketAddr::from((localhost, port));

    println!("Setup server at [{socket_address:?}]");
    // run server
    let server = Server::bind(&socket_address);
    tokio::spawn(server.serve(app.into_make_service()));
    loop {
        // await for a new command received to restart
        println!("Waiting for a HTTP::START command to run the command");
        let mut rx_command = tx_cmd_ref.lock().await.clone().subscribe();
        let cmd = rx_command.recv().await?;
        if cmd != HTTPCommand::Start {
            continue;
        }
        println!("starting command");
        let rx_command = tx_cmd_ref.lock().await.clone().subscribe();
        tokio::spawn(handler_http_command(rx_command, tx_restart.clone()));
        // two futures, app vs received signal
        let received_stop = rx_restart.recv();
        let app = runtime();
        // TODO find a better solution than select! that can kill another task async
        select! {
            res = app => {
                match res {
                    Ok(_) => println!("app completed successfully"),
                    Err(err) => eprintln!("app error: {:?}", err),
                }
            }
            _ = received_stop => {
                println!("received stop successfully");
            }
        }
    }
}

async fn handler_http_command(
    mut rx_command: Receiver<HTTPCommand>,
    tx: Sender<()>,
) -> Result<(), failure::Error> {
    loop {
        let cmd = rx_command.recv().await?;
        println!("Received command: {cmd:?}");
        if cmd == HTTPCommand::Stop {
            tx.send(())?;
        }
    }
}

async fn handler_status() -> Response<Body> {
    // Add your code to terminate your service here.
    let status: bool = false;
    let msg: String = format!("status: {status:?}");
    Response::new(Body::from(msg))
}

async fn handler_stop(tx: Extension<Arc<Mutex<Sender<HTTPCommand>>>>) -> Response<Body> {
    // Add your code to terminate your service here.
    tx.lock().await.send(HTTPCommand::Stop).unwrap();
    let msg = "Service terminated.";
    Response::new(Body::from(msg))
}

async fn handler_start(tx: Extension<Arc<Mutex<Sender<HTTPCommand>>>>) -> Response<Body> {
    // Add your code to restart your service here.
    tx.lock().await.send(HTTPCommand::Start).unwrap();
    let msg = "Service restarted.";
    Response::new(Body::from(msg))
}
