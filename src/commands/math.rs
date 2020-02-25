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
#[min_args(1)]
fn math(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let mut namespace = fasteval::EmptyNamespace;
    let value = fasteval::ez_eval(&args.rest(), &mut namespace);

    match value {
        Ok(value) => {
            let _ = msg.channel_id.say(&ctx.http, value);
            Ok(())
        }
        Err(err) => match err {
            _ => {
                let _ = msg.channel_id.say(&ctx.http, format!("```{:?}```", err));
                Ok(())
            }
        },
    }
}
