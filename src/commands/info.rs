use crate::core::built_info;
use serenity::{
    framework::standard::{
        macros::command,
        CommandResult,
    },
    model::prelude::Message,
    prelude::Context,
};

#[command]
fn about(ctx: &mut Context, msg: &Message) -> CommandResult {
    // TODO: Implment a working way to detect and fill in the current application version, rather than updating it by hand.
    //       (GitHub would work, however if for some reason those differ, it could be problematic.)
    let _ = msg.channel_id.say(&ctx.http, format!("{} (v ``{}``) is a small utility bot, developed by <@77812253511913472>, with help from serenity and it's resources.\n\n\
    There are currently no set plans for this bot", "arzte-bot", "0.0.1"));

    Ok(())
}
