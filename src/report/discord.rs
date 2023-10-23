// Discord bot sends hello world to the designated channel via HTTP
use serenity::async_trait;
use serenity::http::Http;
use serenity::model::channel::Message;
use serenity::model::gateway::{GatewayIntents, Ready};
use serenity::model::prelude::ChannelId;
use serenity::prelude::*;
use serenity::Client;
use std::sync::Arc;

// Bot configuration
struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.to_lowercase().contains("hi") || msg.content.to_lowercase().contains("hello")
        {
            if let Err(e) = msg.channel_id.say(&ctx.http, "hello world!").await {
                log::error!("Error sending message: {:?}", e);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        log::info!("[{}] discord bot client is connected", ready.user.name);
    }
}

pub async fn task_discord_bot(
    receiver: tokio::sync::broadcast::Receiver<String>,
    token: String,
    channel_id: u64,
) -> Result<(), failure::Error> {
    log::info!("Initializing discord bot client");

    // specify intents
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    // setup client http
    let mut discord_bot_client = Client::builder(token, intents)
        .event_handler(Bot)
        .await
        .expect("Error creating client");
    let discord_bot_http = discord_bot_client.cache_and_http.http.clone();
    log::info!("running bot client start()");

    let task_ended = tokio::select! {
        _ = channel_broadcast(channel_id, discord_bot_http, receiver) => "discord_channel_broadcast",
        _ = discord_bot_client.start() => "discord_bot_client"
    };
    Err(failure::err_msg(format!("[{task_ended}] ended early.")))
}

async fn channel_broadcast(
    channel_id: u64,
    http: Arc<Http>,
    mut receiver: tokio::sync::broadcast::Receiver<String>,
) -> Result<(), failure::Error> {
    let channel = ChannelId(channel_id);
    loop {
        let received_string = receiver.recv().await?;
        if let Err(e) = channel.say(&http, received_string).await {
            if let serenity::Error::Http(http_error) = e {
                if http_error.status_code().unwrap() == 401 {
                    log::error!("Error: Discord bot token is invalid or unauthorized.");
                }
            } else {
                log::error!("Error sending message: {:?}", e);
            }
        }
    }
}
