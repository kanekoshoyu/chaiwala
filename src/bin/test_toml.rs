use chaiwala::config as config;

fn main() {
    let config = config::get_config("config.toml");
    println!("{config:?}");
}
