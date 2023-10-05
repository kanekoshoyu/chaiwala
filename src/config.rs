use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub kucoin: KuCoin,
    pub discord: Discord,
    pub behaviour: Behaviour,
}

#[derive(Deserialize, Debug)]
pub struct KuCoin {
    pub api_key: String,
    pub secret_key: String,
    pub passphrase: String,
}

#[derive(Deserialize, Debug)]
pub struct Discord {
    pub api_key: String,
}

#[derive(Deserialize, Debug)]
pub struct Behaviour {
    pub monitor_interval_sec: u32,
    pub usd_cyclic_arbitrage: u32,
}

pub fn from_file(filename: &str) -> Config {
    let toml_str = fs::read_to_string(filename).expect("Failed to read config.toml");
    toml::from_str(&toml_str).unwrap()
}
