use asciimath::{
    eval,
    scope,
    Error::UnknownVariable,
};
use serenity::{
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    model::prelude::Message,
    prelude::Context,
};

#[command]
fn math(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let scope_empty = scope! {};
    let value = eval(&args.rest(), &scope_empty);

    match value {
        Ok(value) => {
            let _ = msg.channel_id.say(&ctx.http, value);
            Ok(())
        }
        Err(err) => match err {
            UnknownVariable(_t) => {
                let _ = msg.channel_id.say(&ctx.http, "Cannot eval with variables");
                Ok(())
            }
            _ => {
                let _ = msg.channel_id.say(&ctx.http, format!("```{:?}```", err));
                Ok(())
            }
        },
    }
}
