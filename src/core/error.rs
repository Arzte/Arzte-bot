use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReactionError {
    #[error("No Guild ID on reaction")]
    NoGuildId,
    #[error("Unable to get lock")]
    LockError,
    #[error("Unable to get container from sharemap")]
    ShareMapGetError,
    #[error("Database Error")]
    DatabaseError(#[from] sqlx::Error),
    #[error("No role found")]
    NoRows,
    #[error("Couldn't find a matching guild member for user id")]
    ErrorFindingGuildMember(#[from] serenity::Error),
    #[error("Error adding role")]
    ErrorAddingRole,
}
