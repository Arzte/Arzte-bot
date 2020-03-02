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
    model::prelude::Message,
    prelude::Context,
};
use std::sync::Arc;

#[command]
#[aliases("pre")]
#[required_permissions(ADMINISTRATOR)]
/// Sets the bot's prefix in a server to whatever argument is provided
/// If no new prefix is provided, it will show the server's current prefix
/// Restricted to Users with the Administrator permission
fn prefix(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let guild_id = msg.guild_id.ok_or("Failed to get server ID")?;
    let (db_pooler, tokio_lock, prefix_hashmap_lock) = {
        let data = ctx.data.try_read().ok_or("Failed to get data lock")?;
        let db_pool = Arc::clone(
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
        (db_pool, tokio_lock, prefix_hashmap_lock)
    };
    let mut db_pool = &db_pooler.pooler;
    let mut tokio = tokio_lock
        .try_lock()
        .ok()
        .ok_or("Failed to get runtime lock")?;
    let mut prefix_hashmap = prefix_hashmap_lock
        .try_lock()
        .ok()
        .ok_or("Failed to get prefix cache")?;

    if args.is_empty() {
        if let Some((_, prefix)) = prefix_hashmap.get_key_value(guild_id.as_u64()) {
            return msg
                .channel_id
                .say(&ctx.http, format!("This guild's prefix is ``{}``", prefix))
                .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()));
        }
        let prefix_option = tokio
            .block_on(
                sqlx::query!(
                    "SELECT prefix FROM guild WHERE id = $1",
                    *guild_id.as_u64() as i64
                )
                .fetch_optional(&mut db_pool),
            )
            .ok()
            .ok_or("Failed to get prefix from database")?;

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

    let guild_name = msg
        .guild(&ctx)
        .ok_or("Failed to get guild")?
        .try_read()
        .ok_or("Failed to get guild lock")?
        .name
        .clone();

    tokio.block_on(
            sqlx::query!(
                "INSERT INTO guild (id, name, prefix) VALUES ($1, $2, $3) ON CONFLICT (id) DO UPDATE SET name = $2, prefix = $3",
                *guild_id.as_u64() as i64,
                guild_name,
                args.rest().to_string()
            )
            .execute(&mut db_pool),
        ).ok().ok_or("Error setting prefix, try again later")?;

    msg.channel_id
        .say(
            &ctx.http,
            format!("Changed the server's prefix to ``{}``", args.rest()),
        )
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
}
