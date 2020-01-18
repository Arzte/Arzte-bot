use crate::core::built_info;
use crate::ShardManagerContainer;
use chrono::Duration;
use serenity::{
    client::bridge::gateway::ShardId,
    framework::standard::{
        macros::command,
        Args,
        CommandError,
        CommandResult,
    },
    model::{
        id::GuildId,
        prelude::Message,
    },
    prelude::Context,
};

#[command]
fn about(ctx: &mut Context, msg: &Message) -> CommandResult {
    // TODO: Implment a working way to detect and fill in the current application version, rather than updating it by hand.
    //       (GitHub would work, however if for some reason those differ, it could be problematic.)
    let _ = msg.channel_id.say(&ctx.http, format!("{} (v ``{}``) is a small utility bot, developed by <@77812253511913472>, with help from serenity and it's resources.", built_info::PKG_NAME, built_info::PKG_VERSION));
    Ok(())
}

#[command]
#[description = "Shows various information about a user"]
#[only_in("guilds")]
fn user(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let guild_id = msg.guild_id.ok_or("Failed to get GuildID from Message.")?;
    let member = if msg.mentions.is_empty() {
        if args.is_empty() {
            msg.member(&ctx).ok_or("Could not find member.")?
        } else {
            (*(guild_id
                .to_guild_cached(&ctx)
                .ok_or("Failed to get Guild from GuildId")?
                .read()
                .members_starting_with(args.rest(), false, true)
                .first()
                .ok_or("Could not find member")?))
            .clone()
        }
    } else {
        guild_id.member(
            &ctx,
            msg.mentions
                .first()
                .ok_or("Failed to get user mentioned.")?,
        )?
    };

    let user = member.user.read();
    let nickname = member.nick.map_or("None".to_owned(), |nick| nick.clone());
    let member_joined = member
        .joined_at
        .map_or("Unavailable".to_owned(), |d| format!("{}", d));

    msg.channel_id
        .send_message(&ctx, move |m| {
            m.embed(move |e| {
                e.author(|a| a.name(&user.name).icon_url(&user.face()))
                    .field("Discriminator", format!("#{:04}", user.discriminator), true)
                    .field("User ID", user.id, true)
                    .field("Nickname", nickname, true)
                    .field("User Created", user.created_at(), true)
                    .field("Joined Server", member_joined, true)
            })
        })
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
}

#[command]
#[description = "Shows various information about a guild."]
#[only_in("guilds")]
fn guild(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = if !args.is_empty() {
        GuildId(args.single::<u64>()?)
    } else if let Some(gid) = msg.guild_id {
        gid
    } else {
        msg.channel_id.say(
            &ctx.http,
            "I was unable to get the current guild id, try again later.",
        )?;
        return Ok(());
    };
    let guild = guild_id
        .to_guild_cached(&ctx)
        .ok_or("Failed to get Guild from GuildID")?
        .read()
        .clone();

    msg.channel_id
        .send_message(&ctx, move |m| {
            m.embed(move |e| {
                e.author(|a| {
                    a.name(&guild.name);
                    if let Some(guild_icon) = &guild.icon_url() {
                        a.icon_url(guild_icon);
                    }
                    a
                })
                .field("Owner", format!("<@{}>", &guild.owner_id.as_u64()), true)
                .field("Guild ID", format!("{}", guild_id), true)
                .field("Members", guild.member_count, true)
                .field("Region", &guild.region, true)
                .field("Roles", &guild.roles.len(), true)
                .field("Emojis", &guild.emojis.len(), true)
                .field("Features", format!("{:?}", guild.features), true)
                .field(
                    "Nitro Boost Level",
                    format!("{:?}", guild.premium_tier),
                    true,
                )
                .field("Nitro Boosts", guild.premium_subscription_count, true);
                if let Some(splash) = guild.splash_url() {
                    e.image(splash);
                }
                e.footer(|f| {
                    f.text(format!(
                        "Guild created on {}",
                        guild_id
                            .created_at()
                            .format("%a, %d %h %Y @ %H:%M:%S")
                            .to_string()
                    ))
                })
            })
        })
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
}

#[command]
fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    let start = msg.timestamp.timestamp_millis();
    let mut message = msg.channel_id.say(&ctx.http, "Pong!")?;
    let timestamp = message.timestamp.timestamp_millis() - start;

    let data = ctx.data.write();

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            let _ = msg.reply(&ctx, "There was a problem getting the shard manager");

            return Ok(());
        }
    };

    let manager = shard_manager.lock();
    let runners = manager.runners.lock();

    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            let _ = msg.reply(&ctx, "No shard found");

            return Ok(());
        }
    };
    let latency = match runner.latency {
        Some(latency) => match Duration::from_std(latency) {
            Ok(milli) => format!("{}ms", milli.num_milliseconds()),
            Err(_error) => "result is to high to calculate.".to_string(),
        },
        None => "0ms".to_string(),
    };

    let string = format!(
        "Pong! \n**```prolog\n   Message Latency: {}ms, \n     Shard Latency: {}\n```**",
        timestamp, latency
    );
    message.edit(&ctx, |m| m.content(string))?;

    Ok(())
}
