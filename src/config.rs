use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug)]
pub struct Config {
    kucoin: KuCoin,
    discord: Discord,
    behaviour: Behaviour,
}

#[derive(Deserialize, Debug)]
pub struct KuCoin {
    api_key: String,
    secret_key: String,
    passphrase: String,
}

#[derive(Deserialize, Debug)]
pub struct Discord {
    api_key: String,
}

#[derive(Deserialize, Debug)]
pub struct Behaviour {
    monitor_interval_sec: u32,
    usd_cyclic_arbitrage: u32,
}

pub fn get_config(filename: &str) -> Config {
    let toml_str = fs::read_to_string(filename).expect("Failed to read config.toml");
    toml::from_str(&toml_str).unwrap()
}
