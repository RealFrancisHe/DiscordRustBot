use serenity::{framework::standard::{macros::command, CommandResult, buckets::RevertBucket, Args}, prelude::Context, model::prelude::Message};




#[command]
#[aliases("kitten")]
#[description = "Sends a cat emoji."]
#[bucket = "emoji"]
#[required_permissions("ADMINISTRATOR")]
async fn cat(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, ":cat:").await?;

    // Return one ticket to bucket
    Err(RevertBucket.into())
}

#[command]
#[aliases("woof")]
#[description = "Sends a dog emoji."]
#[bucket = "emoji"]
async fn dog(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, ":cat:").await?;

    Ok(())
}

#[command]
async fn bird(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let say_content = if args.is_empty() {
        ":bird: can find the animals.".to_string()
    } else {
        format!(":bird: could not find animal named: `{}`.", args.rest())
    };

    msg.channel_id.say(&ctx.http, say_content).await?;

    Ok(())
}
