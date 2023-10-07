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
        if msg.content == "!hello" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "world!").await {
                println!("Error sending message: {:?}", e);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("[{}] discord bot client is connected", ready.user.name);
    }
}

pub async fn task_discord_bot(
    receiver: tokio::sync::broadcast::Receiver<String>,
    token: String,
    channel_id: u64,
) -> Result<(), failure::Error> {
    println!("Initializing discord bot client");

    // specify intents
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    // setup client http
    let mut discord_bot_client = Client::builder(token, intents)
        .event_handler(Bot)
        .await
        .expect("Error creating client");
    let discord_bot_http = discord_bot_client.cache_and_http.http.clone();

    // assign your channel

    // TODO fix below
    println!("spawned channe_broadcast()");
    // do not await below
    tokio::spawn(channel_broadcast(channel_id, discord_bot_http, receiver));
    println!("running bot client start()");
    discord_bot_client.start().await.unwrap();
    println!("should not arrive here!");
    Ok(())
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
                    eprintln!("Error: Discord bot token is invalid or unauthorized.");
                }
            } else {
                eprintln!("Error sending message: {:?}", e);
            }
        }
    }
}
