use crate::commands::emoji::*;
use crate::commands::functions::*;
use crate::commands::math::*;
use crate::commands::owner::*;
use dotenv::dotenv;
use serenity::client::bridge::gateway::ShardId;
use serenity::{
    framework::{
        standard::{
            buckets::LimitedFor,
            macros::{check, command, group},
            Args, CommandOptions, CommandResult, Reason,
        },
        StandardFramework,
    },
    http::Http,
    model::{
        prelude::{Channel, Message},
        Permissions,
    },
    prelude::{Context, GatewayIntents},
    utils::{content_safe, ContentSafeOptions},
    Client,
};
use std::fmt::Write;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

mod commands;

#[group]
#[commands(
    about,
    am_i_admin,
    say,
    commands,
    multiply,
    ping,
    latency,
    about_roles,
    wakeup
)]
struct General;

#[group]
#[prefix = "math"]
#[commands(multiply)]
struct Math;

#[group]
#[prefixes("emoji", "em")] // multiple prefiexs (same thing)
#[description = "Grouped emoji command responses"]
// #[summary = "Emoji stuffs"]
#[default_command(bird)]
#[commands(cat, dog)]
struct Emoji;

#[group]
#[owners_only]
#[only_in(guilds)]
// #[summary = "Server Owner Commands"]
#[commands(slow_mode)]
struct Owner;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Tracing
    // tracing_subscriber::fmt::init();

    // Configuration
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let http = Http::new(&token);

    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("No bot id {:?}", why),
            }
        }
        Err(why) => panic!("No App info {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| {
            c.with_whitespace(true)
                .on_mention(Some(bot_id))
                .prefix("/")
                .delimiters(vec![" ", " "]) // include space
                .owners(owners)
        })
        .before(before)
        .after(after)
        .unrecognised_command(unknown_command)
        // .normal_message(normal_message)
        .on_dispatch_error(dispatch_error)
        .bucket("emoji", |b| b.delay(3))
        .await
        .bucket("complicated", |b| {
            b.limit(2)
                .time_span(30)
                .delay(3)
                .limit_for(LimitedFor::Channel)
                .await_ratelimits(1)
                .delay_action(delay_action)
        })
        .await
        .help(&MY_HELP)
        .group(&GENERAL_GROUP)
        .group(&MATH_GROUP)
        .group(&EMOJI_GROUP)
        .group(&OWNER_GROUP);

    // the monitored resources
    let intents = GatewayIntents::all();
    // Logging in as bot
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .type_map_insert::<CommandCounter>(HashMap::default())
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    // listener
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

// Creates the command using this macro
#[command]
// Uses complicated bucket
#[bucket = "complicated"]
async fn commands(ctx: &Context, msg: &Message) -> CommandResult {
    let mut contents = "Commands used:\n".to_string();

    let data = ctx.data.read().await;
    let counter = data
        .get::<CommandCounter>()
        .expect("Expected CommandCounter in TypeMap.");

    for (k, v) in counter {
        writeln!(contents, "- {name}: {amount}", name = k, amount = v)?;
    }

    msg.channel_id.say(&ctx.http, &contents).await?;

    Ok(())
}

// Repeats what user passed as argument
#[command]
async fn say(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    match args.single_quoted::<String>() {
        Ok(x) => {
            let settings = if let Some(guild_id) = msg.guild_id {
                ContentSafeOptions::default()
                    .clean_channel(false)
                    .display_as_member_from(guild_id)
            } else {
                ContentSafeOptions::default()
                    .clean_channel(false)
                    .clean_role(false)
            };

            let content = content_safe(&ctx.cache, x, &settings, &msg.mentions);

            msg.channel_id.say(&ctx.http, &content).await?;

            return Ok(());
        }
        Err(_) => {
            msg.reply(ctx, "An argument is require to run this command.")
                .await?;
            return Ok(());
        }
    };
}

// Check function to see if caller is owner
#[check]
#[name = "Owner"]
async fn owner_check(
    _: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> Result<(), Reason> {
    let id = std::env::var("ID")
        .expect("Expected an ID in the environment")
        .parse::<u64>()
        .expect("ID is not parseable as i32");
    if msg.author.id != id {
        return Err(Reason::User("Lacked owner permission".to_string()));
    }

    Ok(())
}

#[command]
async fn some_long_command(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id
        .say(&ctx.http, &format!("Arguments: {:?}", args.rest()))
        .await?;

    Ok(())
}

#[command]
// #[allowed_roles("waaah wheres my higher role")]
async fn about_roles(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let potential_role_name = args.rest();

    if let Some(guild) = msg.guild(&ctx.cache) {
        if let Some(role) = guild.role_by_name(potential_role_name) {
            if let Err(why) = msg
                .channel_id
                .say(&ctx.http, &format!("Role-ID: {}", role.id))
                .await
            {
                println!("Error sending message: {:?}", why);
            }

            return Ok(());
        }
    }

    msg.channel_id
        .say(
            &ctx.http,
            format!("Could not find role named: {:?}", potential_role_name),
        )
        .await?;

    Ok(())
}

#[command]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(&ctx.http, "This is Masterawes's bot! :)")
        .await?;

    Ok(())
}

#[command]
async fn latency(ctx: &Context, msg: &Message) -> CommandResult {
    // shard manager
    let data = ctx.data.read().await;

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            msg.reply(ctx, "There was a problem getting the shard manager")
                .await?;

            return Ok(());
        }
    };

    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;

    // Information about the shard runner
    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            msg.reply(ctx, "No shard found").await?;

            return Ok(());
        }
    };

    msg.reply(ctx, &format!("The shard latency is {:?}", runner.latency))
        .await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(Owner)]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong!").await?;

    Ok(())
}

#[command]
async fn am_i_admin(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    if let Some(member) = &msg.member {
        for role in &member.roles {
            if role
                .to_role_cached(&ctx.cache)
                .map_or(false, |r| r.has_permission(Permissions::ADMINISTRATOR))
            {
                msg.channel_id
                    .say(&ctx.http, "Yes, you are an admin.")
                    .await?;

                return Ok(());
            }
        }
    }

    msg.channel_id
        .say(&ctx.http, "No, you are not an admin.")
        .await?;

    Ok(())
}

#[command]
async fn slow_mode(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let say_content = if let Ok(slow_mode_rate_seconds) = args.single::<u64>() {
        if let Err(why) = msg
            .channel_id
            .edit(&ctx.http, |c| c.rate_limit_per_user(slow_mode_rate_seconds))
            .await
        {
            println!("Error setting channel's slow mode rate: {:?}", why);

            format!(
                "Failed to set slow mode to `{}` seconds.",
                slow_mode_rate_seconds
            )
        } else {
            format!(
                "Successfully set slow mode rate to `{}` seconds.",
                slow_mode_rate_seconds
            )
        }
    } else if let Some(Channel::Guild(channel)) = msg.channel_id.to_channel_cached(&ctx.cache) {
        let slow_mode_rate = channel.rate_limit_per_user.unwrap_or(0);
        format!("Current slow mode rate is `{}` seconds.", slow_mode_rate)
    } else {
        "Failed to find channel in cache.".to_string()
    };

    msg.channel_id.say(&ctx.http, say_content).await?;

    Ok(())
}

// Sub-commands
#[command("upper")]
#[sub_commands(sub)]
async fn upper_command(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.reply(&ctx.http, "This is the main function!").await?;

    Ok(())
}

// Sub-command of `upper`
#[command]
#[aliases("subcommand", "sub-command", "secret")]
#[description("This is `upper`'s sub-command.")]
async fn sub(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.reply(&ctx.http, "This is the sub function!").await?;

    Ok(())
}
