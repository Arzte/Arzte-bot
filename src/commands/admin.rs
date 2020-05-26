use crate::core::structs::{
    PoolContainer,
    PrefixHashMapContainer,
    TokioContainer,
};
use serenity::{
    framework::standard::{
        macros::command,
        Args,
        CommandError,
        CommandResult,
    },
    model::prelude::{
        Message,
        MessageId,
        RoleId,
    },
    prelude::Context,
};
use std::sync::Arc;

#[command]
#[aliases("pre")]
#[required_permissions(MANAGE_ROLES)]
/// Sets the bot's prefix in a server to whatever argument is provided
/// If no new prefix is provided, it will show the server's current prefix
/// Restricted to Users with the Administrator permission
fn prefix(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let guild_id = msg.guild_id.ok_or("Failed to get server ID")?;
    let (fancy_db, tokio_lock, prefix_hashmap_lock) = {
        let data = ctx.data.try_read().ok_or("Failed to get data lock")?;
        let fancy_db = Arc::clone(
            data.get::<PoolContainer>()
                .ok_or("Failed to get database pool out of data")?,
        );
        let tokio_lock = Arc::clone(
            data.get::<TokioContainer>()
                .ok_or("Failed to get runtime")?,
        );
        let prefix_hashmap_lock = Arc::clone(
            data.get::<PrefixHashMapContainer>()
                .ok_or("Failed to get prefix cache lock")?,
        );
        (fancy_db, tokio_lock, prefix_hashmap_lock)
    };

    // Limit the scope of the prefix hashmap lock
    {
        let mut prefix_hashmap = prefix_hashmap_lock
            .try_lock()
            .ok()
            .ok_or("Failed to get prefix cache")?;

        if args.is_empty() {
            if let Some(prefix) = prefix_hashmap.get(guild_id.as_u64()) {
                return msg
                    .channel_id
                    .say(&ctx.http, format!("This guild's prefix is ``{}``", prefix))
                    .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()));
            }
            let prefix_option = {
                let mut tokio = tokio_lock
                    .try_lock()
                    .ok()
                    .ok_or("Failed to get runtime lock")?;
                tokio
                    .block_on(
                        sqlx::query!(
                            "SELECT prefix FROM guild WHERE id = $1",
                            *guild_id.as_u64() as i64
                        )
                        .fetch_optional(fancy_db.pool()),
                    )
                    .ok()
                    .ok_or("Failed to get prefix from database")?
            };

            if let Some(result) = prefix_option {
                prefix_hashmap.insert(*guild_id.as_u64(), result.prefix.clone());
                return msg
                    .channel_id
                    .say(
                        &ctx.http,
                        format!("This servers prefix is ``{}``", result.prefix),
                    )
                    .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()));
            } else {
                return msg
                    .channel_id
                    .say(&ctx.http, "This servers  prefix is ``a.``")
                    .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()));
            }
        }

        prefix_hashmap.insert(*guild_id.as_u64(), args.rest().to_string());
    }

    let guild_name = msg
        .guild(&ctx)
        .ok_or("Failed to get guild")?
        .try_read()
        .ok_or("Failed to get guild lock")?
        .name
        .clone();

    // At this point we don't need args anymore, and can just reassign it as a String.
    let args = args.rest().to_owned();

    // Limit scope of tokio lock
    {
        let mut tokio = tokio_lock.try_lock()?;
        tokio.block_on(
            sqlx::query!(
                "INSERT INTO guild (id, name, prefix) VALUES ($1, $2, $3) ON CONFLICT (id) DO UPDATE SET name = $2, prefix = $3",
                *guild_id.as_u64() as i64,
                guild_name,
                args
            )
            .execute(fancy_db.pool()),
        ).ok().ok_or("Error setting prefix, try again later")?;
    }

    msg.channel_id
        .say(
            &ctx.http,
            format!("Changed the server's prefix to ``{}``", args),
        )
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
}

#[command]
// #[aliases("")]
#[min_args(3)]
#[required_permissions(ADMINISTRATOR)]
/// Sets the bot's prefix in a server to whatever argument is provided
/// If no new prefix is provided, it will show the server's current prefix
/// Restricted to Users with the Administrator permission
fn reaction(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let emoji_string = args.single::<String>()?;
    let emoji_str: &str = emoji_string.as_ref();
    let emoji = {
        match serenity::utils::parse_emoji(emoji_str) {
            Some(emoji) => Some(emoji),
            None => {
                if emoji_str.starts_with("<a:") {
                    let mut split = emoji_str.split(":");
                    let name = split.nth(1).ok_or("Failed to parse emoji")?;
                    log::debug!("emoji name: {}", name);
                    let id = split
                        .nth(0)
                        .ok_or("Failed to get name of emoji")?
                        .trim_end_matches(">");
                    log::debug!("emoji id: {}", id);
                    Some(serenity::model::misc::EmojiIdentifier {
                        id: serenity::model::id::EmojiId(id.parse::<u64>()?),
                        name: name.to_string(),
                    })
                } else if !emoji_str.is_ascii() {
                    None
                } else {
                    return Err(CommandError(format!("Error parsing emoji")));
                }
            }
        }
    };
    let role_id = {
        match args.single::<RoleId>() {
            Ok(role_id) => role_id,
            Err(_) => RoleId(
                serenity::utils::parse_role(args.single::<String>()?)
                    .ok_or("Couldn't parse role id")?,
            ),
        }
    };
    let message_id = MessageId(args.single::<u64>()?);
    let guild_id = msg.guild_id.ok_or("Couldn't get guild id")?;

    let (fancy_db, runtime_lock) = {
        let data = ctx.data.read();
        let fancy_db = Arc::clone(data.get::<PoolContainer>().ok_or("Couldn't get fancy db")?);
        let runtime_lock = Arc::clone(
            data.get::<TokioContainer>()
                .ok_or("Couldn't get runtime lock")?,
        );
        (fancy_db, runtime_lock)
    };

    {
        let mut tokio = runtime_lock.try_lock()?;
        if let Some(emoji_indentifier) = emoji {
            log::debug!(
                "Custom Emoji: {}:{}",
                emoji_indentifier.name,
                emoji_indentifier.id
            );
            tokio.block_on(
                sqlx::query!("INSERT INTO reaction_roles (guild_id, role_id, message_id, emoji_id, name) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (role_id) DO UPDATE SET guild_id = $1, role_id = $2, message_id = $3, emoji_id = $4, name = $5",
                        *guild_id.as_u64() as i64,
                        *role_id.as_u64() as i64,
                        *message_id.as_u64() as i64,
                        *emoji_indentifier.id.as_u64() as i64,
                        emoji_indentifier.name
                    ).execute(fancy_db.pool())
            )?;
        } else {
            log::debug!("Unicode emoji: {}", emoji_str);
            tokio.block_on(sqlx::query!("INSERT INTO reaction_roles (guild_id, role_id, message_id, name) VALUES ($1, $2, $3, $4) ON CONFLICT (role_id) DO UPDATE SET guild_id = $1, role_id = $2, message_id = $3, name = $4",
                        *guild_id.as_u64() as i64,
                        *role_id.as_u64() as i64,
                        *message_id.as_u64() as i64,
                        emoji_str
                    ).execute(fancy_db.pool())
            )?;
        }
    }

    return msg
        .channel_id
        .say(&ctx.http, format!("Successfully added the role `{}`, with the emoji {}, to the message:\nhttps://discordapp.com/channels/{}/{}/{}", role_id.to_role_cached(&ctx).ok_or("Successfully added role, however unable to find role name in cache.")?.name, emoji_str, guild_id.as_u64(), msg.channel_id.as_u64(), message_id))
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()));
}
