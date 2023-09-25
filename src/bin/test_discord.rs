use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::model::channel::Message;
use serenity::model::gateway::GatewayIntents;
use serenity::framework::StandardFramework;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        // Check if the message is from the desired channel
        let channel_id = 1;
        if msg.channel_id == channel_id {
            // Check if the message content is "!hi" (or any other trigger you prefer)
            if msg.content == "!hi" {
                // Send the "hi" message
                if let Err(why) = msg.channel_id.say(&ctx.http, "Hi!").await {
                    println!("Error sending message: {:?}", why);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let token = "YOUR_BOT_TOKEN_HERE";

    // Adjust intents as needed
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILDS; 

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(StandardFramework::new())
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
