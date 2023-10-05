use chaiwala::config as config;

fn main() {
    let config = config::from_file("config.toml");
    println!("{config:?}");
}
