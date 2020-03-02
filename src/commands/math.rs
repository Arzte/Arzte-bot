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
    utils::{
        content_safe,
        ContentSafeOptions,
    },
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
            fastevalError::Undefined(variable) => {
                let content = content_safe(&ctx, variable, &ContentSafeOptions::default());
                msg.channel_id
                    .say(
                        &ctx.http,
                        format!(
                            "Unknown variable: `{}`\nPS: Variables are unsupported",
                            content
                        ),
                    )
                    .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
            }
            _ => {
                let content =
                    content_safe(&ctx, format!("{:#?}", err), &ContentSafeOptions::default());
                msg.channel_id
                    .say(&ctx.http, content)
                    .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
            }
        },
    }
}

#[command]
#[min_args(1)]
/// For when math isn't precise enough for you. (15 second timeout on calculations)
fn precision_math(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let args_full = args.rest();

    let value = if args_full.contains("for") {
        String::from("Illegal Character ``for``")
    } else if args_full.contains("print") {
        String::from("Illegal Character ``print``")
    } else if args_full.contains("warrenty") {
        String::from("Illegal Character ``warrenty``")
    } else if args_full.contains("while") {
        String::from("Illegal Character ``while``")
    } else {
        match bc::bc_timeout!(args_full) {
            Ok(value) => {
                if value.len() < 2000 {
                    value
                } else {
                    String::from("Output too large to send")
                }
            }
            Err(err) => format!("```{:?}```", err),
        }
    };

    let content = content_safe(&ctx, value, &ContentSafeOptions::default());

    msg.channel_id
        .say(&ctx.http, content)
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
}
