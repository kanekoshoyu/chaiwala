use chaiwala::event;
use chaiwala::event::RuntimeStatus;
use chaiwala::webserver;
use tokio::sync::broadcast;
#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // logging format
    kucoin_arbitrage::logger::log_init();
    log::info!("Log setup");

    // build our application with a single route
    let (tx_status, rx_status) = broadcast::channel::<event::RuntimeStatus>(2);

    let mut taskpool = tokio::task::JoinSet::new();
    taskpool.spawn(webserver::task_api_router(tx_status));
    taskpool.spawn(recv_status(rx_status));
    taskpool.join_next().await;
    Ok(())
}

async fn recv_status(
    mut rx_status: broadcast::Receiver<RuntimeStatus>,
) -> Result<(), failure::Error> {
    loop {
        let res = rx_status.recv().await?;
        log::info!("recv_status receievd: [{res:?}]");
    }
}
