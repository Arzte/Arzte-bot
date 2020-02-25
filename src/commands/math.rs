use fasteval::error::Error as fastevalError;
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

#[command]
#[min_args(1)]
/// For fun with math, does not currently support variables*
/// *(There are plans to change that in the future!)
fn math(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let mut namespace = fasteval::EmptyNamespace;
    let value = fasteval::ez_eval(&args.rest(), &mut namespace);

    match value {
        Ok(value) => msg
            .channel_id
            .say(&ctx.http, value)
            .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(())),

        Err(err) => match err {
            fastevalError::Undefined(variable) => msg
                .channel_id
                .say(
                    &ctx.http,
                    format!(
                        "Unknown variable: `{}`\nPS: Variables are unsupported",
                        variable
                    ),
                )
                .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(())),
            _ => msg
                .channel_id
                .say(&ctx.http, format!("```{:?}```", err))
                .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(())),
        },
    }
}

#[command]
#[min_args(1)]
/// For when math isn't precise enough for you. (15 second timeout on calculations)
fn precision_math(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let args_full = args.rest();
    if args_full.contains("for") {
        return msg
            .channel_id
            .say(&ctx.http, "Illegal Character ``for``")
            .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()));
    } else if args_full.contains("print") {
        return msg
            .channel_id
            .say(&ctx.http, "Illegal Character ``print``")
            .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()));
    } else if args_full.contains("warrenty") {
        return msg
            .channel_id
            .say(&ctx.http, "Illegal Character ``print``")
            .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()));
    }

    let value = bc::bc_timeout!(args_full);

    match value {
        Ok(value) => msg
            .channel_id
            .say(&ctx.http, value)
            .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(())),

        Err(err) => msg
            .channel_id
            .say(&ctx.http, format!("```{:?}```", err))
            .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(())),
    }
}
