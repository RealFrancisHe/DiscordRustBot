use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serenity::client::bridge::gateway::ShardManager;
use serenity::framework::standard::{Args, HelpOptions, CommandGroup, CommandResult, help_commands, DispatchError};
use serenity::framework::standard::macros::{help, hook};
use serenity::model::gateway::Ready;
use serenity::model::prelude::UserId;
use serenity::prelude::TypeMapKey;
use serenity::utils::MessageBuilder;
use serenity::{async_trait, prelude::{EventHandler, Context}, model::prelude::Message};
use tokio::sync::Mutex;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct CommandCounter;

impl TypeMapKey for CommandCounter {
    type Value = HashMap<String, u64>;
}
pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // handles events

    async fn message(&self, ctx: Context, msg: Message ) {
        let body = msg.content;
        if &body == "!ping" {
            if let Err(failure) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", failure);
            }
        } else if &body == "!messageme" {
            let dm = 
                msg.author.dm(&ctx, |m| m.content("Hello!")).await;

            if let Err(why) = dm {
                println!("Error when direct messaging user: {:?}", why);
            }
        } else if &body == "!builder" {
            let channel = match msg.channel_id.to_channel(&ctx).await {
                Ok(channel) => channel,
                Err(why) => {
                    println!("Error getting channel: {:?}", why);

                    return;
                },
            };

            let response = MessageBuilder::new()
                .push("User ")
                .push_bold_safe(&msg.author.name)
                .push(" pinged ")
                .mention(&channel)
                .push(" channel")
                .build();

            if let Err(why) = msg.channel_id.say(&ctx.http, &response).await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name)
    }
}

#[help]
#[individual_command_tip = "Hello"]
#[command_not_found_text = "Could not find: `{}`."]
#[max_levenshtein_distance(3)] // how similar the strings have to be
#[indention_prefix = "+"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Nothing"]
#[wrong_channel = "Strike"]
pub async fn my_help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(ctx, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[hook]
pub async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    println!("Got command '{}' by the user '{}'", command_name, msg.author.name);

    let mut data = ctx.data.write().await;
    let counter = data.get_mut::<CommandCounter>().expect("Expected CommandCounter in TypeMap.");
    let entry = counter.entry(command_name.to_string()).or_insert(0);
    *entry += 1;

    true
}

#[hook]
pub async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => println!("Processed command '{}'", command_name),
        Err(why) => println!("Command '{}' return an error: {:?}", command_name, why),
        }
}

#[hook]
pub async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    println!("Could not find command '{}'", unknown_command_name);
}

#[hook]
pub async fn normal_message(_ctx: &Context, msg: &Message) {
    println!("Message is not a command '{}'", msg.content)
}

#[hook]
pub async fn delay_action(ctx: &Context, msg: &Message) {
    let _ = msg.react(ctx, '⏱').await;
}

#[hook]
pub async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError, _command_name: &str) {
    if let DispatchError::Ratelimited(info) = error {
        // We notify them only once.
        if info.is_first_try {
            let _ = msg
                .channel_id
                .say(&ctx.http, &format!("Try this again in {} seconds.", info.as_secs()))
                .await;
        }
    }
}



