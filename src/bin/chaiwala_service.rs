/// Executes triangular arbitrage, as a service
use chaiwala::report::counter::task_log_mps;
use chaiwala::report::discord::task_discord_bot;
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
use kucoin_arbitrage::monitor::counter;
use kucoin_arbitrage::strategy::all_taker_btc_usd::task_pub_chance_all_taker_btc_usd;
use std::sync::Arc;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::broadcast::{channel, Receiver, Sender};
use tokio::sync::Mutex;
use tokio::task::JoinSet;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // logging format
    kucoin_arbitrage::logger::log_init();
    log::info!("Log setup");

    // credentials
    let config = chaiwala::config::from_file("config.toml")?;

    // channels
    let (tx_discord_message, rx_discord_message) = channel::<String>(256);
    let (tx_runtime_status, rx_runtime_status) = channel::<RuntimeStatus>(5);

    // setup http server
    let msg = tokio::select! {
        res = task_signal_handle() => format!("received external signal ({res:?})"),
        res = core_runtime(config.clone(), tx_discord_message, rx_runtime_status) => format!("core ended ({res:?})"),
        res = service(config, rx_discord_message, tx_runtime_status) => format!("service ended ({res:?})"),
    };
    log::info!("{msg}, Good bye!");
    Ok(())
}
async fn service(
    config: chaiwala::config::Config,
    rx_discord_message: Receiver<String>,
    tx_runtime_status: Sender<RuntimeStatus>,
) -> Result<(), failure::Error> {
    let discord_bot_token: String = config.discord.token.clone();
    let discord_channel_id: u64 = config.discord.channel_id;

    let mut taskpool_service = tokio::task::JoinSet::new();

    // TODO setup http server here
    taskpool_service.spawn(task_discord_bot(
        rx_discord_message,
        discord_bot_token,
        discord_channel_id,
    ));
    taskpool_service.join_next().await.unwrap()?
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum RuntimeStatus {
    Running,
    Idle,
}

async fn received_runtime_status(
    mut rx_runtime_status: Receiver<RuntimeStatus>,
    status: RuntimeStatus,
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
    tx_discord_message: Sender<String>,
    rx_runtime_status: Receiver<RuntimeStatus>,
) -> Result<(), failure::Error> {
    // TODO discord message sender to be used by chaiwala's custom core::taskpool_infrastructure::task_log_mps
    loop {
        received_runtime_status(rx_runtime_status.resubscribe(), RuntimeStatus::Running).await?;
        // change status as start
        let message: String = tokio::select! {
            _ = core(config.clone().core(), tx_discord_message.clone()) => String::from("unexpected end of core"),
            res = received_runtime_status(rx_runtime_status.resubscribe(), RuntimeStatus::Idle) =>format!("signal[{:?}]", res)

        };
        // change status as stop
    }
}
async fn core(
    config: kucoin_arbitrage::config::Config,
    tx_discord_message: Sender<String>,
) -> Result<(), failure::Error> {
    // TODO setup reporting with tx_discord_message

    // config parameters
    let budget = config.behaviour.usd_cyclic_arbitrage;
    let monitor_interval = config.behaviour.monitor_interval_sec;

    // system mps counters
    let api_input_counter = Arc::new(Mutex::new(counter::Counter::new("api_input")));
    let best_price_counter = Arc::new(Mutex::new(counter::Counter::new("best_price")));
    let chance_counter = Arc::new(Mutex::new(counter::Counter::new("chance")));
    let order_counter = Arc::new(Mutex::new(counter::Counter::new("order")));

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
    // for syncing public orderbook
    let (tx_orderbook, rx_orderbook) = channel::<OrderbookEvent>(1024 * 2);
    // for getting notable orderbook after syncing
    let (tx_orderbook_best, rx_orderbook_best) = channel::<OrderbookEvent>(512);
    // for getting chance
    let (tx_chance, rx_chance) = channel::<ChanceEvent>(64);
    // for placing order
    let (tx_order, rx_order) = channel::<OrderEvent>(16);
    // for getting private order changes
    let (tx_orderchange, rx_orderchange) = channel::<OrderChangeEvent>(128);
    log::info!("Broadcast channels setup");

    // local orderbook
    let full_orderbook = Arc::new(Mutex::new(FullOrderbook::new()));
    log::info!("Local empty full orderbook setup");

    // infrastructure tasks
    let mut taskpool_infrastructure = JoinSet::new();
    taskpool_infrastructure.spawn(task_sync_orderbook(
        rx_orderbook,
        tx_orderbook_best,
        full_orderbook.clone(),
        api_input_counter.clone(),
    ));
    taskpool_infrastructure.spawn(task_pub_chance_all_taker_btc_usd(
        rx_orderbook_best,
        tx_chance,
        full_orderbook.clone(),
        hash_symbols,
        budget as f64,
        best_price_counter.clone(),
    ));
    taskpool_infrastructure.spawn(task_gatekeep_chances(
        rx_chance,
        rx_orderchange,
        tx_order,
        chance_counter.clone(),
    ));
    taskpool_infrastructure.spawn(task_place_order(
        rx_order,
        api.clone(),
        order_counter.clone(),
    ));
    taskpool_infrastructure.spawn(task_log_mps(
        tx_discord_message,
        vec![
            api_input_counter.clone(),
            best_price_counter.clone(),
            chance_counter.clone(),
            order_counter.clone(),
        ],
        monitor_interval as u64,
    ));

    // collect all initial orderbook states with REST
    task_get_initial_orderbooks(api.clone(), symbol_infos, full_orderbook).await?;
    log::info!("Aggregated all the symbols");
    let mut taskpool_subscription = JoinSet::new();
    // publishes OrderChangeEvent from private subscription
    taskpool_subscription.spawn(task_pub_orderchange_event(api.clone(), tx_orderchange));
    // publishes OrderBookEvent from public subscription
    for (i, sub) in subs.iter().enumerate() {
        taskpool_subscription.spawn(task_pub_orderbook_event(
            api.clone(),
            sub.to_vec(),
            tx_orderbook.clone(),
        ));
        log::info!("{i:?}-th session of WS subscription setup");
    }

    // terminate if taskpools failed
    let message = tokio::select! {
        res = taskpool_infrastructure.join_next() =>
            format!("infrastructure task pool error [{res:?}]"),
        res = taskpool_subscription.join_next() => format!("subscription task pool error [{res:?}]"),
    };
    Err(failure::err_msg(format!("unexpected error [{message}]")))
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
