use chaiwala::event;
use chaiwala::webserver;
use tokio::sync::broadcast;
#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // logging format
    kucoin_arbitrage::logger::log_init();
    log::info!("Log setup");

    // build our application with a single route
    let (tx_status, _) = broadcast::channel::<event::RuntimeStatus>(2);
    webserver::task_api_router(tx_status).await
}
