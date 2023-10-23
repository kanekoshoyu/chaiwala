/// Executes triangular arbitrage, as a service
use chaiwala::event;
use chaiwala::report::counter::task_log_mps;
use chaiwala::report::discord::task_discord_bot;
use chaiwala::webserver::task_api_router;
use kucoin_api::client::{Kucoin, KucoinEnv};
use kucoin_arbitrage::broker::gatekeeper::kucoin::task_gatekeep_chances;
use kucoin_arbitrage::broker::order::kucoin::task_place_order;
use kucoin_arbitrage::broker::orderbook::internal::task_sync_orderbook;
use kucoin_arbitrage::broker::orderbook::kucoin::{
    task_get_initial_orderbooks, task_pub_orderbook_event,
};
use kucoin_arbitrage::broker::orderchange::kucoin::task_pub_orderchange_event;
use kucoin_arbitrage::broker::symbol::filter::{symbol_with_quotes, vector_to_hash};
use kucoin_arbitrage::broker::symbol::kucoin::{format_subscription_list, get_symbols};
use kucoin_arbitrage::event::{
    chance::ChanceEvent, order::OrderEvent, orderbook::OrderbookEvent,
    orderchange::OrderChangeEvent,
};
use kucoin_arbitrage::model::orderbook::FullOrderbook;
use kucoin_arbitrage::monitor::counter::Counter;
use kucoin_arbitrage::monitor::task::task_monitor_channel_mps;
use kucoin_arbitrage::strategy::all_taker_btc_usd::task_pub_chance_all_taker_btc_usd;
use std::sync::Arc;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::broadcast::{channel, Receiver, Sender};
use tokio::sync::Mutex;
use tokio::task::JoinSet;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // logging format
    chaiwala::logger::log_init()?;
    log::info!("Log setup");

    // credentials
    let config = chaiwala::config::from_file("config.toml")?;

    // channels
    let tx_discord_message = channel::<String>(256).0;
    let tx_runtime_status = channel::<event::RuntimeStatus>(5).0;

    // setup http server
    let msg = tokio::select! {
        _ = task_signal_handle() => format!("received external signal"),
        res = core_runtime(config.clone(), tx_runtime_status.subscribe(), tx_discord_message.clone()) => format!("core ended ({res:?})"),
        res = service(config.clone(), tx_discord_message.subscribe(), tx_runtime_status.clone()) => format!("service ended ({res:?})"),
    };
    log::info!("{msg}, Good bye!");
    Ok(())
}
async fn service(
    config: chaiwala::config::Config,
    rx_discord_message: Receiver<String>,
    tx_runtime_status: Sender<event::RuntimeStatus>,
) -> Result<(), failure::Error> {
    let discord_bot_token: String = config.discord.token.clone();
    let discord_channel_id: u64 = config.discord.channel_id;

    let mut taskpool_service = tokio::task::JoinSet::new();

    taskpool_service.spawn(task_discord_bot(
        rx_discord_message,
        discord_bot_token,
        discord_channel_id,
    ));
    taskpool_service.spawn(task_api_router(tx_runtime_status));
    taskpool_service.join_next().await.unwrap()?
}

async fn received_runtime_status(
    mut rx_runtime_status: Receiver<event::RuntimeStatus>,
    status: event::RuntimeStatus,
) -> Result<(), failure::Error> {
    loop {
        if let Ok(res) = rx_runtime_status.recv().await {
            log::info!("Received runtime status request [{res:?}]");
            if res.eq(&status) {
                return Ok(());
            }
        }
    }
}

async fn core_runtime(
    config: chaiwala::config::Config,
    rx_runtime_status: Receiver<event::RuntimeStatus>,
    tx_discord_message: Sender<String>,
) -> Result<(), failure::Error> {
    loop {
        log::info!("waiting for start command");
        received_runtime_status(
            rx_runtime_status.resubscribe(),
            event::RuntimeStatus::Running,
        )
        .await?;
        let message: String = tokio::select! {
            res = core(config.clone().core(), tx_discord_message.clone()) => {
                if let Err(e) = res{
                    format!("Core received error [{e}]")
                }else{
                    String::from("unexpected end of core")
                }
            },
            res = received_runtime_status(rx_runtime_status.resubscribe(), event::RuntimeStatus::Idle) =>format!("signal[{:?}]", res)

        };
        log::info!("Runtime set to Idle [{message}]");
    }
}

async fn core(
    config: kucoin_arbitrage::config::Config,
    tx_discord_message: Sender<String>,
) -> Result<(), failure::Error> {
    // config parameters
    let budget = config.behaviour.usd_cyclic_arbitrage;
    let monitor_interval = config.behaviour.monitor_interval_sec;

    // API endpoints
    let api = Kucoin::new(KucoinEnv::Live, Some(config.kucoin_credentials()))?;
    log::info!("Credentials setup");

    // get all symbols concurrently
    let symbol_list = get_symbols(api.clone()).await;
    log::info!("Total exchange symbols: {:?}", symbol_list.len());

    // filter with either btc or usdt as quote
    let symbol_infos = symbol_with_quotes(&symbol_list, "BTC", "USDT");
    let hash_symbols = Arc::new(Mutex::new(vector_to_hash(&symbol_infos)));
    log::info!("Total symbols in scope: {:?}", symbol_infos.len());

    // list subscription using the filtered symbols
    let subs = format_subscription_list(&symbol_infos);
    log::info!("Total orderbook WS sessions: {:?}", subs.len());

    // create broadcast channels

    // system mps counters
    let cx_orderbook = Arc::new(Mutex::new(Counter::new("orderbook")));
    let tx_orderbook = channel::<OrderbookEvent>(1024 * 2).0;
    let cx_orderbook_best = Arc::new(Mutex::new(Counter::new("best_price")));
    let tx_orderbook_best = channel::<OrderbookEvent>(512).0;
    let cx_chance = Arc::new(Mutex::new(Counter::new("chance")));
    let tx_chance = channel::<ChanceEvent>(64).0;
    let cx_order = Arc::new(Mutex::new(Counter::new("order")));
    let tx_order = channel::<OrderEvent>(16).0;
    let cx_orderchange = Arc::new(Mutex::new(Counter::new("orderchange")));
    let tx_orderchange = channel::<OrderChangeEvent>(128).0;
    log::info!("Broadcast channels setup");

    // local orderbook
    let full_orderbook = Arc::new(Mutex::new(FullOrderbook::new()));
    log::info!("Local empty full orderbook setup");

    // infrastructure tasks
    let mut taskpool_infrastructure: JoinSet<Result<(), failure::Error>> = JoinSet::new();
    taskpool_infrastructure.spawn(task_sync_orderbook(
        tx_orderbook.subscribe(),
        tx_orderbook_best.clone(),
        full_orderbook.clone(),
    ));
    taskpool_infrastructure.spawn(task_pub_chance_all_taker_btc_usd(
        tx_orderbook_best.subscribe(),
        tx_chance.clone(),
        full_orderbook.clone(),
        hash_symbols,
        budget as f64,
    ));
    taskpool_infrastructure.spawn(task_gatekeep_chances(
        tx_chance.subscribe(),
        tx_orderchange.subscribe(),
        tx_order.clone(),
    ));
    taskpool_infrastructure.spawn(task_place_order(tx_order.subscribe(), api.clone()));

    // monitor tasks
    let mut taskpool_monitor = JoinSet::new();
    taskpool_monitor.spawn(task_monitor_channel_mps(
        tx_orderbook.subscribe(),
        cx_orderbook.clone(),
    ));
    taskpool_monitor.spawn(task_monitor_channel_mps(
        tx_orderbook_best.subscribe(),
        cx_orderbook_best.clone(),
    ));
    taskpool_monitor.spawn(task_monitor_channel_mps(
        tx_chance.subscribe(),
        cx_chance.clone(),
    ));
    taskpool_monitor.spawn(task_monitor_channel_mps(
        tx_order.subscribe(),
        cx_order.clone(),
    ));
    taskpool_monitor.spawn(task_monitor_channel_mps(
        tx_orderchange.subscribe(),
        cx_orderchange.clone(),
    ));
    taskpool_monitor.spawn(task_log_mps(
        tx_discord_message,
        vec![
            cx_orderbook.clone(),
            cx_orderbook_best.clone(),
            cx_chance.clone(),
            cx_order.clone(),
            cx_orderchange.clone(),
        ],
        monitor_interval as u64,
    ));

    // Initial orderbook states from REST
    task_get_initial_orderbooks(api.clone(), symbol_infos, full_orderbook).await?;
    log::info!("Aggregated all the symbols");

    // websocket subscription tasks
    let mut taskpool_subscription = JoinSet::new();
    // publishes OrderChangeEvent from private API
    taskpool_subscription.spawn(task_pub_orderchange_event(api.clone(), tx_orderchange));
    // publishes OrderBookEvent from public API
    for (i, sub) in subs.iter().enumerate() {
        taskpool_subscription.spawn(task_pub_orderbook_event(
            api.clone(),
            sub.to_vec(),
            tx_orderbook.clone(),
        ));
        log::info!("{i:?}-th session of WS subscription setup");
    }

    // terminate if any taskpool failed
    let message: String = tokio::select! {
        res = taskpool_infrastructure.join_next() =>
            format!("Infrastructure task pool error [{res:?}]"),
        res = taskpool_monitor.join_next() =>
            format!("Monitor task pool error [{res:?}]"),
        res = taskpool_subscription.join_next() => format!("Subscription task pool error [{res:?}]"),
    };
    Err(failure::err_msg(message))
}

/// wait for any external terminating signal
async fn task_signal_handle() -> Result<(), failure::Error> {
    let mut sigterm = signal(SignalKind::terminate()).unwrap();
    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    tokio::select! {
        _ = sigterm.recv() => exit_program("SIGTERM").await?,
        _ = sigint.recv() => exit_program("SIGINT").await?,
    };
    Ok(())
}

/// handle external signal
async fn exit_program(signal_alias: &str) -> Result<(), failure::Error> {
    log::info!("Received [{signal_alias}] signal");
    Ok(())
}
