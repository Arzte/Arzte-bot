use crate::{
    PoolContainer,
    TokioContainer,
};
use serenity::{
    model::prelude::{
        Reaction,
        ReactionType,
    },
    prelude::Context,
};
use std::sync::Arc;

/// Adds a preset role to a user based on a reaction add event in a guild,
/// that matches the reaction in the DB, provided the it's added in the same
/// guild
pub fn reaction_add(ctx: &Context, reaction_add: &Reaction) -> Option<()> {
    let guild_id = *reaction_add.guild_id?.as_u64() as i64;
    let message_id = *reaction_add.message_id.as_u64() as i64;

    let role_id = {
        let (fancy_db, runtime_lock) = {
            let data = ctx.data.try_read()?;
            let fancy_db = Arc::clone(data.get::<PoolContainer>()?);
            let runtime_lock = Arc::clone(data.get::<TokioContainer>()?);
            (fancy_db, runtime_lock)
        };

        match reaction_add.emoji {
            ReactionType::Custom { id, .. } => {
                let data = {
                    let mut runtime = runtime_lock.try_lock().ok()?;
                    runtime
                    .block_on(
                        sqlx::query!(
                            "SELECT role_id FROM reaction_roles WHERE guild_id = $1 AND message_id = $2 AND emoji_id = $3",
                            guild_id,
                            message_id,
                            *id.as_u64() as i64
                        )
                        .fetch_optional(fancy_db.pool()),
                    )
                    .ok()?
                };
                data?.role_id
            }
            ReactionType::Unicode(ref name) => {
                let data = {
                    let mut runtime = runtime_lock.try_lock().ok()?;
                    runtime
                    .block_on(
                        sqlx::query!(
                            "SELECT role_id FROM reaction_roles WHERE guild_id = $1 AND message_id = $2 AND name = $3",
                            guild_id,
                            message_id,
                            name
                        )
                        .fetch_optional(fancy_db.pool()),
                    )
                    .ok()?
                };
                data?.role_id
            }
            _ => return None,
        }
    };

    let mut guild_member = {
        let guild = reaction_add.guild_id?;
        guild.member(ctx, reaction_add.user_id).ok()?
    };

    guild_member.add_role(ctx, role_id as u64).ok()
}
