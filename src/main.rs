use std::env;

use ping::Handler;
use serenity::{prelude::GatewayIntents, Client};

mod ping;

#[tokio::main]
async fn main() {
    // Configuration
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in env");

    // the monitored resources
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    
    // Logging in as bot
    let mut client = Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    // listener
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
