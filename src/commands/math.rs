use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::Message,
    prelude::Context,
};

#[command]
#[aliases("*")] // Allows for ~math * as well as ~math multiply
async fn multiply(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let first = args.single::<f64>()?;
    let second = args.single::<f64>()?;

    let res = first * second;

    msg.channel_id.say(&ctx.http, &res.to_string()).await?;

    Ok(())
}
