use serenity::model::prelude::UserId;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::{Channel, ChannelId, ChannelType, GuildChannel, Message},
    prelude::Context,
};
use tokio::time::{sleep, Duration};

use super::functions::ShardManagerContainer;

#[command]
#[owners_only]
async fn quit(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    if let Some(manager) = data.get::<ShardManagerContainer>() {
        msg.reply(ctx, "Shutting down!").await?;
        manager.lock().await.shutdown_all().await;
    } else {
        msg.reply(ctx, "There was a problem getting the shard manager")
            .await?;

        return Ok(());
    }

    Ok(())
}

#[command]
async fn wakeup(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // Getting UserId
    let user_id = match args.single::<UserId>() {
        Ok(id) => id,
        Err(_) => return Ok(()),
    };

    // Get list of channel ids
    let mut channel_ids = Vec::new();
    if let Some(guild) = msg.guild(&ctx.cache) {
        let mut original: Option<ChannelId> = None;
        // Check if user in voice channel
        if let Some(voice_state) = guild.voice_states.get(&user_id) {
            if let Some(starting) = voice_state.channel_id {
                original = Some(starting);
            }
        } else {
            msg.channel_id
                .say(&ctx.http, &format!("{} not in a Voice Channel", user_id))
                .await?;
            return Ok(());
        }
        // Get list of voice channels
        for channel in guild.channels.values() {
            if let Channel::Guild(GuildChannel { kind: n, .. }) = channel {
                if *n == ChannelType::Voice {
                    channel_ids.push(channel);
                }
            }
        }

        // Shuffle through channel
        for channel in channel_ids {
            let _member = guild.move_member(&ctx.http, user_id, channel).await;
            sleep(Duration::from_millis(250)).await;
        }

        if let Some(val) = original {
            let _move = guild.move_member(&ctx.http, user_id, val).await;
        }
    }

    // temp
    msg.channel_id
        .say(&ctx.http, &format!("Woke {} up", user_id))
        .await?;

    Ok(())
}
