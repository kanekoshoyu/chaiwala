// Discord bot sends hello world to the designated channel via HTTP
use chaiwala::config;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::{GatewayIntents, Ready};
use serenity::model::prelude::ChannelId;
use serenity::prelude::*;
use serenity::Client;

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
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let config = config::from_file("config.toml");
    let discord_bot_token: String = config.discord.token;

    // specify intents
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    // setup client http
    let mut discord_bot_client = Client::builder(discord_bot_token, intents)
        .event_handler(Bot)
        .await
        .expect("Error creating client");
    let discord_bot_http = discord_bot_client.cache_and_http.http.clone();

    // assign your channel
    let channel_id = ChannelId(config.discord.channel_id);

    // say hello world to the channel
    let msg = "Arbitrage bot has initialized!";
    if let Err(e) = channel_id.say(&discord_bot_http, msg).await {
        if let serenity::Error::Http(http_error) = e {
            if http_error.status_code().unwrap() == 401 {
                eprintln!("Error: Discord bot token is invalid or unauthorized.");
            }
        } else {
            eprintln!("Error sending message: {:?}", e);
        }
    }

    // start is a blocking function that does not stop.
    // spawn for actual use
    if let Err(why) = discord_bot_client.start().await {
        println!("Client error: {:?}", why);
    }
}
