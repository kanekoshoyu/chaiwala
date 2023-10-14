use kucoin_arbitrage::error::Error;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Config {
    pub kucoin: kucoin_arbitrage::config::KuCoin,
    pub behaviour: kucoin_arbitrage::config::Behaviour,
    pub discord: Discord,
}
impl Config {
    pub fn core(self) -> kucoin_arbitrage::config::Config {
        kucoin_arbitrage::config::Config {
            kucoin: self.kucoin,
            behaviour: self.behaviour,
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Discord {
    pub token: String,
    pub channel_id: u64,
}

pub fn from_file(filename: &str) -> Result<Config, Error> {
    let toml_str = fs::read_to_string(filename).map_err(Error::IoError)?;
    toml::from_str(&toml_str).map_err(Error::TomlError)
}
