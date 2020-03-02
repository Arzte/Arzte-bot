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
#[aliases("version", "v")]
/// Tells some information about the bot
fn about(ctx: &mut Context, msg: &Message) -> CommandResult {
    let bot_owner = UserId(77_812_253_511_913_472).to_user(&ctx)?;
    let _ = msg.channel_id.say(
        &ctx.http,
        format!(
            "<@{}> version {}, is developed by {} with help from serenity and its resources.\nSource code can be found at https://github.com/Arzte/Arzte-bot",
            ctx.cache.read().user.id,
            built_info::PKG_VERSION,
            bot_owner.name
        ),
    );
    Ok(())
}

#[command]
/// Shows the avatar for the user or specified user.
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
#[only_in("guilds")]
#[aliases("u")]
/// Shows various information about a user
fn user(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let guild_id = msg.guild_id.ok_or("Failed to get GuildID from Message.")?;
    // TODO: Find a user via userid if provided
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
                e.timestamp(msg.timestamp.to_rfc3339());
                e.footer(|f| {
                    f.text(format!("Requested by {}", msg.author.tag()));
                    f.icon_url(msg.author.face());
                    f
                });
                e
            })
        })
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
}

#[command]
#[only_in("guilds")]
#[aliases("g", "s", "guild")]
/// Shows various information about a guild.
fn server(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
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
        .ok_or("No server with this Guild ID can be found")?
        .read()
        .clone();
    let roles: String = {
        let role_hash = &guild.roles;
        let mut role_id_list = String::new();
        let mut role_vec: Vec<_> = role_hash
            .iter()
            .filter(|(_role_id, role)| !role.managed)
            .collect();
        // This sorts the vector by position, in reverse order. (so @@everyone is the last item)
        role_vec.sort_unstable_by(|(_role_id_a, role_a), (_role_id_b, role_b)| {
            role_a
                .position
                .partial_cmp(&role_b.position)
                .unwrap()
                .reverse()
        });
        if role_vec.len() < 60 {
            for (role_id, _role) in role_vec.iter() {
                role_id_list.push_str(format!("<@&{}> ", role_id.as_u64()).as_ref())
            }
        } else {
            // Take will panic if there isn't as many items in a iter as
            // the number you try to take, therefore it can only be done when the
            // the items we're itering over is known to be equal to or above the amount
            // we're trying to take
            for (role_id, _role) in role_vec.iter().take(60) {
                role_id_list.push_str(format!("<@&{}> ", role_id.as_u64()).as_ref())
            }
            role_id_list.push_str("*Unable to show all roles.*");
        }
        role_id_list
    };

    msg.channel_id
        .send_message(&ctx, move |m| {
            m.embed(move |e| {
                e.author(|a| a.name(&guild.name));
                if let Some(guild_icon) = &guild.icon_url() {
                    e.thumbnail(guild_icon);
                };
                e.field("Owner", format!("<@{}>", &guild.owner_id.as_u64()), true);
                e.field("Guild ID", format!("{}", guild_id), true);
                e.field("Members", guild.member_count, true);
                e.field(
                    "Guild created on",
                    guild_id
                        .created_at()
                        .format("%A, %d %B %Y \n%H:%M:%S UTC")
                        .to_string(),
                    true,
                );
                e.field("Region", &guild.region, true);
                e.field("Emojis", &guild.emojis.len(), true);
                e.field("Channels", guild.channels.len(), true);
                if !guild.features.is_empty() {
                    e.field("Features", guild.features.join(", "), true);
                }
                e.field(
                    "Nitro Boost Level",
                    format!("{:?}", guild.premium_tier),
                    true,
                );
                e.field("Nitro Boosts", guild.premium_subscription_count, true);
                e.field("Roles", roles, false);
                if let Some(splash) = guild.splash_url() {
                    e.image(splash);
                }
                if let Some(description) = guild.description {
                    e.description(description);
                }
                e.timestamp(msg.timestamp.to_rfc3339());
                e.footer(|f| {
                    f.text(format!("Requested by {}", msg.author.tag()));
                    f.icon_url(msg.author.face());
                    f
                });
                e
            })
        })
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
}

#[command]
#[aliases("p")]
/// Does a quick test to find out the latancy of Discord Relative to the bot
fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    let start = chrono::offset::Utc::now().timestamp_millis();
    let mut message = msg.channel_id.say(&ctx.http, "Pong!")?;
    let message_latency = {
        let millis = message.timestamp.timestamp_millis() - start;
        if millis > 0 {
            millis
        } else {
            0
        }
    };

    let latency = {
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
        message_latency, latency
    );
    message.edit(&ctx, |m| m.content(string))?;

    Ok(())
}
