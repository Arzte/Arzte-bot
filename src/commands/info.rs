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
    let _ = msg.channel_id.say(&ctx.http, format!("{} (v ``{}``) is a small utility bot, developed by <@77812253511913472>, with help from serenity and it's resources.\n\n\
    There are currently no set plans for this bot", built_info::PKG_NAME, built_info::PKG_VERSION));
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
fn guild(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = if args.is_empty() {
        if let Some(gid) = msg.guild_id {
            *gid.as_u64()
        } else {
            msg.channel_id.say(
                &ctx.http,
                "I was unable to get the current guild id, try again later.",
            )?;
            return Ok(());
        }
    } else {
        args.single::<u64>()?
    };
    // Why ask the API before the Cache?
    // The idea is to lazily check if the bot can access the info as well as ensuring there *is* some data in the cache for the guild.
    // This also ensures the data that'll be displayed is the most accurate, in case any of the info isn't already up to date in the cache
    let g = match GuildId(guild_id).to_partial_guild(&ctx.http) {
        Ok(partial_guild) => partial_guild,
        Err(_arg_error) => {
            msg.channel_id
                .say(&ctx.http, ":no_entry_sign: Invalid server/Not Available")?;
            return Ok(());
        }
    };
    let guild_lock = &ctx.cache.read().guild(&g.id);
    if let Some(guild_lock) = guild_lock {
        let guildd = guild_lock.read();
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    let mut e = e
                        .title(&g.name)
                        .field("ID", &g.id, false)
                        .field("Name", &g.name, true)
                        .field("Owner", format!("<@{}>", &g.owner_id.as_u64()), true)
                        .field("Region", &g.region, true)
                        .field("Members", guildd.members.len(), true)
                        .field(
                            "Created on",
                            &g.id
                                .created_at()
                                .format("%a, %d %h %Y @ %H:%M:%S")
                                .to_string(),
                            true,
                        )
                        .field("Roles", &g.roles.len(), true)
                        .field("Emojis", &g.emojis.len(), true);
                    if let Some(icon_url) = &g.icon_url() {
                        e = e
                            .thumbnail(&icon_url)
                            .author(|a| a.name(&g.name).icon_url(&icon_url));
                    }

                    e
                })
            })
            .unwrap();
    };
    Ok(())
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
