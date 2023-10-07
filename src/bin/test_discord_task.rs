// Discord bot sends hello world to the designated channel via HTTP

use tokio::sync::broadcast::channel;
use tokio::time::{sleep, Duration};

/// TODO turn into async task

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    let config: chaiwala::config::Config = chaiwala::config::from_file("config.toml")?;
    let discord_bot_token: String = config.discord.token;
    let channel_id: u64 = config.discord.channel_id;
    let (sender, receiver) = channel::<String>(256);

    tokio::spawn(chaiwala::report::discord::task_discord_bot(
        discord_bot_token,
        channel_id,
        receiver,
    ));
    let duration = Duration::from_secs(10);

    let mut counter = 0;
    loop {
        let message = format!("counter: {counter}");
        counter += 1;
        let _ = sender.send(message).unwrap();
        sleep(duration).await;
    }
}
