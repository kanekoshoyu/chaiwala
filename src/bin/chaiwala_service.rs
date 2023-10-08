use chaiwala::report::discord::task_discord_bot;
/// Executes triangular arbitrage
// api
use kucoin_api::{
    client::{Kucoin, KucoinEnv},
    model::market::OrderBookType,
    model::websocket::{WSTopic, WSType},
};
// tasks
use kucoin_arbitrage::broker::gatekeeper::kucoin::task_gatekeep_chances;
use kucoin_arbitrage::broker::order::kucoin::task_place_order;
use kucoin_arbitrage::broker::orderbook::kucoin::{task_pub_orderbook_event, task_sync_orderbook};
use kucoin_arbitrage::broker::orderchange::kucoin::task_pub_orderchange_event;
use kucoin_arbitrage::broker::symbol::filter::{symbol_with_quotes, vector_to_hash};
use kucoin_arbitrage::broker::symbol::kucoin::{format_subscription_list, get_symbols};
use kucoin_arbitrage::strategy::all_taker_btc_usd::task_pub_chance_all_taker_btc_usd;
// events
use kucoin_arbitrage::event::chance::ChanceEvent;
use kucoin_arbitrage::event::order::OrderEvent;
use kucoin_arbitrage::event::orderbook::OrderbookEvent;
use kucoin_arbitrage::event::orderchange::OrderChangeEvent;
// models
use kucoin_arbitrage::model::counter::Counter;
use kucoin_arbitrage::model::orderbook::FullOrderbook;
use kucoin_arbitrage::translator::traits::OrderBookTranslator;
// async system
use std::sync::Arc;
use tokio::sync::broadcast::channel;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // Provides logging format
    kucoin_arbitrage::logger::log_init();
    log::info!("Log setup");

    // Declares all the system counters
    let api_input_counter = Arc::new(Mutex::new(Counter::new("api_input")));
    let best_price_counter = Arc::new(Mutex::new(Counter::new("best_price")));
    let chance_counter = Arc::new(Mutex::new(Counter::new("chance")));
    let order_counter = Arc::new(Mutex::new(Counter::new("order")));

    // Reads config.toml
    let config = chaiwala::config::from_file("config.toml")?;
    let discord_bot_token: String = config.discord.token.clone();
    let discord_channel_id: u64 = config.discord.channel_id;
    let core_config = config.core();
    let monitor_interval = core_config.behaviour.monitor_interval_sec;
    let budget = core_config.behaviour.usd_cyclic_arbitrage;

    // Kucoin API Endpoints
    let api: Kucoin = Kucoin::new(KucoinEnv::Live, Some(core_config.kucoin_credentials()))?;
    let url_public = api.clone().get_socket_endpoint(WSType::Public).await?;
    let url_private = api.clone().get_socket_endpoint(WSType::Private).await?;
    log::info!("Credentials setup");

    // Gets all symbols concurrently
    let symbol_list = get_symbols(api.clone()).await;
    log::info!("Total exchange symbols: {:?}", symbol_list.len());

    // Filters with either btc or usdt as quote
    let symbol_infos = symbol_with_quotes(&symbol_list, "BTC", "USDT");
    let hash_symbols = Arc::new(Mutex::new(vector_to_hash(&symbol_infos)));
    log::info!("Total symbols in scope: {:?}", symbol_infos.len());

    // Changes a list of SymbolInfo into a 2D list of WSTopic per session in max 100 index
    let subs = format_subscription_list(&symbol_infos);
    log::info!("Total orderbook WS sessions: {:?}", subs.len());

    // Creates broadcast channels
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
    // for reporting to Discord
    let (tx_discord_message, rx_discord_message) = channel::<String>(256);
    log::info!("Broadcast channels setup");

    // Creates local orderbook
    let orderbooks = Arc::new(Mutex::new(FullOrderbook::new()));
    log::info!("Local orderbook setup");

    // Infrastructure tasks
    // USD cyclic arbitrage budget obtained from CONFIG
    tokio::spawn(task_sync_orderbook(
        rx_orderbook,
        tx_orderbook_best,
        orderbooks.clone(),
        api_input_counter.clone(),
    ));
    tokio::spawn(task_pub_chance_all_taker_btc_usd(
        rx_orderbook_best,
        tx_chance,
        orderbooks.clone(),
        hash_symbols,
        budget as f64,
        best_price_counter.clone(),
    ));
    tokio::spawn(task_gatekeep_chances(
        rx_chance,
        rx_orderchange,
        tx_order,
        chance_counter.clone(),
    ));
    tokio::spawn(task_place_order(
        rx_order,
        api.clone(),
        order_counter.clone(),
    ));
    tokio::spawn(task_discord_bot(
        rx_discord_message,
        discord_bot_token,
        discord_channel_id,
    ));

    // Gather all the orderbooks concurrently
    let symbols: Vec<String> = symbol_infos.into_iter().map(|info| info.symbol).collect();
    gather_orderbook_with_rest(symbols, api.clone(), orderbooks).await;

    // TODO revert the flow, we should first setup the infrastructure, then setup the data flow

    // Subscribes public orderbook WS per session, this is the source of data for the infrastructure tasks
    for (i, sub) in subs.iter().enumerate() {
        let mut ws_public = api.websocket();
        ws_public.subscribe(url_public.clone(), sub.clone()).await?;
        // TODO change to task_pub_orderbook_event
        tokio::spawn(task_pub_orderbook_event(ws_public, tx_orderbook.clone()));
        log::info!("{i:?}-th session of WS subscription setup");
    }

    // Subscribes private order change websocket
    let mut ws_private = api.websocket();
    ws_private
        .subscribe(url_private.clone(), vec![WSTopic::TradeOrders])
        .await?;
    tokio::spawn(task_pub_orderchange_event(ws_private, tx_orderchange));

    log::info!("All application tasks setup");

    // Background routine
    chaiwala::report::counter::system_monitor_task(
        tx_discord_message,
        vec![
            api_input_counter.clone(),
            best_price_counter.clone(),
            chance_counter.clone(),
            order_counter.clone(),
        ],
        monitor_interval as u64,
    )
    .await?;
    panic!("Program should not arrive here")
}

async fn gather_orderbook_with_rest(
    symbols: Vec<String>,
    api: Kucoin,
    orderbooks: Arc<Mutex<FullOrderbook>>,
) {
    let tasks: Vec<_> = symbols
        .iter()
        .map(|symbol| {
            // clone variables per task before spawn
            let api = api.clone();
            let orderbooks_refcopy = orderbooks.clone();
            let symbol = symbol.clone();

            tokio::spawn(async move {
                loop {
                    // log::info!("Obtaining initial orderbook[{}] from REST", symbol);
                    let res = api.get_orderbook(&symbol, OrderBookType::L100).await;
                    if res.is_err() {
                        log::warn!("orderbook[{}] did not respond, retry", &symbol);
                        continue;
                    }
                    let res = res.unwrap().data;
                    if res.is_none() {
                        log::warn!("orderbook[{}] received none, retry", &symbol);
                        continue;
                    }
                    let data = res.unwrap();
                    // log::info!("Initial sequence {}:{}", &symbol, data.sequence);
                    let mut x = orderbooks_refcopy.lock().await;
                    x.insert(symbol.to_string(), data.to_internal());
                    break;
                }
            })
        })
        .collect();
    futures::future::join_all(tasks).await;
    log::info!("Collected all the symbols");
}
