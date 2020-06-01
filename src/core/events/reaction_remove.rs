use crate::{
    core::error::ReactionError,
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
pub fn reaction_remove(ctx: &Context, removed_reaction: &Reaction) -> Result<(), ReactionError> {
    let guild_id = *removed_reaction
        .guild_id
        .ok_or(ReactionError::NoGuildId)?
        .as_u64() as i64;
    let message_id = *removed_reaction.message_id.as_u64() as i64;

    let role_id = {
        let (fancy_db, runtime_lock) = {
            let data = ctx.data.try_read().ok_or(ReactionError::LockError)?;
            let fancy_db = Arc::clone(
                data.get::<PoolContainer>()
                    .ok_or(ReactionError::ShareMapGetError)?,
            );
            let runtime_lock = Arc::clone(
                data.get::<TokioContainer>()
                    .ok_or(ReactionError::ShareMapGetError)?,
            );
            (fancy_db, runtime_lock)
        };

        match removed_reaction.emoji {
            ReactionType::Custom { id, .. } => {
                let data = {
                    let mut runtime = runtime_lock
                        .try_lock()
                        .map_err(|_| ReactionError::LockError)?;
                    runtime
                        .block_on(
                            sqlx::query!(
                                "SELECT role_id FROM reaction_roles WHERE guild_id = $1 AND message_id = $2 AND emoji_id = $3",
                                guild_id,
                                message_id,
                                *id.as_u64() as i64
                            )
                            .fetch_optional(fancy_db.pool()),
                        )?
                };
                data.ok_or(ReactionError::NoRows)?.role_id
            }
            ReactionType::Unicode(ref name) => {
                let data = {
                    let mut runtime = runtime_lock
                        .try_lock()
                        .map_err(|_| ReactionError::LockError)?;
                    runtime
                        .block_on(
                            sqlx::query!(
                                "SELECT role_id FROM reaction_roles WHERE guild_id = $1 AND message_id = $2 AND name = $3",
                                guild_id,
                                message_id,
                                name
                            )
                            .fetch_optional(fancy_db.pool()),
                        )?
                };
                data.ok_or(ReactionError::NoRows)?.role_id
            }
            _ => return Ok(()), // We don't know reaction type this is, so we ignore it.
        }
    };

    let mut guild_member = {
        let guild = removed_reaction.guild_id.ok_or(ReactionError::NoGuildId)?;
        guild.member(ctx, removed_reaction.user_id)?
    };

    guild_member
        .remove_role(ctx, role_id as u64)
        .map_err(|_| ReactionError::ErrorAddingRole)
}
