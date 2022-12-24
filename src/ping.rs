use serenity::model::gateway::Ready;
use serenity::{async_trait, prelude::{EventHandler, Context}, model::prelude::Message};



pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // handles events

    async fn message(&self, ctx: Context, msg: Message ) {
        if msg.content == "!ping" {
            if let Err(failure) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", failure);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name)
    }
}