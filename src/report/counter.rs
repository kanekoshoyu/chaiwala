use std::{sync::Arc, time::Duration};

use kucoin_arbitrage::{global::counter_helper, model::counter::Counter};
use tokio::sync::broadcast::Sender;
use tokio::sync::Mutex;
use tokio::time::sleep;

async fn report_counter_mps(
    tx: Sender<String>,
    counters: Vec<Arc<Mutex<Counter>>>,
    interval: u64,
) -> Result<(), failure::Error> {
    log::info!("Reporting broadcast data rate");
    let mut status = String::new();
    for counter in counters.iter() {
        let (name, count) = {
            let p = counter.lock().await;
            (p.name, p.data_count)
        };
        let line = format!("{name:?}: {count:?} points ({:?}mps)", count / interval);
        log::info!("{line}");
        status += &line;
        status += "\n";
        // clear the data
        counter_helper::reset(counter.clone()).await;
    }
    tx.send(status)?;
    Ok(())
}

pub async fn system_monitor_task(
    tx: Sender<String>,
    counters: Vec<Arc<Mutex<Counter>>>,
    interval: u64,
) -> Result<(), failure::Error> {
    let monitor_delay = Duration::from_secs(interval);
    loop {
        sleep(monitor_delay).await;
        report_counter_mps(tx.clone(), counters.clone(), interval)
            .await
            .expect("report status error");
    }
}
