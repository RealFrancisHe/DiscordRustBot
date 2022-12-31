use serenity::{framework::standard::{CommandResult, macros::command, Command, Args}, model::prelude::Message, prelude::Context};

use super::functions::ShardManagerContainer;



#[command]
#[owners_only]
async fn quit(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    if let Some(manager) = data.get::<ShardManagerContainer>() {
        msg.reply(ctx, "Shutting down!").await?;
        manager.lock().await.shutdown_all().await;
    } else {
        msg.reply(ctx, "There was a problem getting the shard manager").await?;

        return Ok(());
    }

    Ok(())
}

// #[command]
// async fn wakeup(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // moves users multiple times through channels
// }