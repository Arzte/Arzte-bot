use crate::core::built_info;
use crate::ShardManagerContainer;
use chrono::Duration;
#[allow(unused_imports)]
use log::{
    error,
    info,
    trace,
    warn,
};
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
        id::UserId,
        prelude::Message,
    },
    prelude::Context,
};

#[command]
#[description = "Tells you about the bot"]
#[aliases("version", "v")]
fn about(ctx: &mut Context, msg: &Message) -> CommandResult {
    let bot_owner = UserId(77_812_253_511_913_472).to_user(&ctx)?;
    let _ = msg.channel_id.say(
        &ctx.http,
        format!(
            "{} {}, developed by {}, with help from serenity and its resources.",
            built_info::PKG_NAME,
            built_info::PKG_VERSION,
            bot_owner.name
        ),
    );
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
    let roles = member.roles(&ctx).map_or(
        "No role data found for this user in the cache".to_owned(),
        |m| {
            let mut role_id_list = String::new();
            for role in m {
                role_id_list.push_str(format!("<@&{}>\n", role.id.as_u64()).as_ref())
            }
            role_id_list
        },
    );
    let nickname = member.nick.map_or("None".to_owned(), |nick| nick);
    let member_joined = member.joined_at.map_or("Unavailable".to_owned(), |d| {
        d.format("%a, %d %h %Y @ %H:%M:%S").to_string()
    });

    msg.channel_id
        .send_message(&ctx, move |m| {
            m.embed(move |e| {
                e.author(|a| a.name(&user.name));
                e.thumbnail(&user.face());
                e.field("Discriminator", format!("#{:04}", user.discriminator), true);
                e.field("User ID", user.id, true);
                e.field("Nickname", nickname, true);
                e.field("Profile", format!("<@{}>", user.id), true);
                e.field("Joined Server", member_joined, true);
                e.field("Roles", roles, true);
                if user.bot {
                    e.field("Bot Account", user.bot, true);
                }
                e.footer(|f| {
                    f.text(format!(
                        "Requested {}",
                        user.created_at()
                            .format("%a, %d %h %Y @ %H:%M:%S")
                            .to_string()
                    ))
                })
            })
        })
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
}

#[command]
#[description = "Shows the avatar for the user or specified user."]
fn avatar(context: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let face = if msg.mentions.is_empty() {
        if args.is_empty() {
            msg.author.face()
        } else {
            let result: Result<String, Box<dyn std::error::Error>> = try {
                msg.guild_id
                    .ok_or("Failed to get GuildId from Message")?
                    .to_guild_cached(&context)
                    .ok_or("Failed to get Guild from GuildId")?
                    .read()
                    .members_starting_with(args.rest(), false, true)
                    .first()
                    .ok_or("Could not find member")?
                    .user_id()
                    .to_user(&context)?
                    .face()
            };
            match result {
                Ok(face) => face,
                Err(e) => {
                    error!("While searching for user: {}", e);
                    msg.author.face()
                }
            }
        }
    } else {
        msg.mentions[0].face()
    };
    msg.channel_id
        .send_message(&context, |m| m.embed(|e| e.image(face)))
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
#[description = "Does a quick test to find out the latancy of Discord Relative to the bot"]
fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    let start = chrono::offset::Utc::now().timestamp_millis();
    let mut message = msg.channel_id.say(&ctx.http, "Pong!")?;
    let timestamp = message.timestamp.timestamp_millis() - start;

    let shard_manager = {
        let data = match ctx.data.try_read() {
            Some(v) => v,
            None => {
                error!("Error getting data lock, trying again...");
                match ctx.data.try_read() {
                    Some(v) => v,
                    None => {
                        error!("Can't get data lock");

                        return Ok(());
                    }
                }
            }
        };
        match data.get::<ShardManagerContainer>() {
            Some(v) => std::sync::Arc::clone(v),
            None => {
                let _ = msg.reply(&ctx, "There was a problem getting the shard manager");

                return Ok(());
            }
        }
    };

    let latency = {
        let manager = shard_manager
            .try_lock()
            .ok_or("Couldn't get a lock on the shard manager")?;
        let runners = manager
            .runners
            .try_lock()
            .ok_or("Couldn't get a lock on the current shard runner")?;

        let shard = match runners.get(&ShardId(ctx.shard_id)) {
            Some(runner) => runner,
            None => {
                let _ = msg.reply(&ctx, "No shard found");

                return Ok(());
            }
        };

        match shard.latency {
            Some(latency) => match Duration::from_std(latency) {
                Ok(milli) => format!("{}ms", milli.num_milliseconds()),
                Err(_error) => "result is to high to calculate.".to_string(),
            },
            None => "TBD".to_string(),
        }
    };

    let string = format!(
        "Pong! \n**```prolog\nMessage Latency: {}ms, \n  Shard Latency: {}\n```**",
        timestamp, latency
    );
    message.edit(&ctx, |m| m.content(string))?;

    Ok(())
}
