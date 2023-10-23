use kucoin_arbitrage::monitor::counter;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::Sender;
use tokio::sync::Mutex;
use tokio::time::sleep;

/// log counters
async fn log_mps(
    tx_message: Sender<String>,
    counters: Vec<Arc<Mutex<counter::Counter>>>,
    interval: u64,
) -> Result<(), kucoin_api::failure::Error> {
    let title = String::from("Broadcast channel data rate");
    log::info!("{title}");
    let mut discord_message = format!("{title}\n");
    for counter in counters.iter() {
        let (name, count) = {
            let p = counter.lock().await;
            (p.name, p.data_count)
        };
        let message = format!("{name:10}: {count:5} points ({:5}mps)", count / interval);
        log::info!("{message}");
        discord_message += &format!("{message}\n");
        // clear the data
        counter::reset(counter.clone()).await;
    }
    tx_message.send(discord_message)?;
    Ok(())
}
/// log counters as a task
pub async fn task_log_mps(
    tx_message: Sender<String>,
    counters: Vec<Arc<Mutex<counter::Counter>>>,
    interval: u64,
) -> Result<(), kucoin_api::failure::Error> {
    let monitor_delay = Duration::from_secs(interval);
    loop {
        sleep(monitor_delay).await;
        log_mps(tx_message.clone(), counters.clone(), interval)
            .await
            .expect("report status error");
    }
}
