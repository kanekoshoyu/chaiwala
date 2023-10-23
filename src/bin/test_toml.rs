use chaiwala::config;

fn main() {
    let config = config::from_file("config.toml");
    log::info!("{config:?}");
}
