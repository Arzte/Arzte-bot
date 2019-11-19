use crate::ShardManagerContainer;
// TODO reenable sentry
// use sentry::Hub;
use serenity::{
    framework::standard::{
        macros::command,
        CommandResult,
    },
    model::prelude::Message,
    prelude::Context,
};
use std::time::Duration;

#[command]
fn quit(ctx: &mut Context, msg: &Message) -> CommandResult {
    let data = ctx.data.write();

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            let _ = msg.reply(&ctx, "There was a problem getting the shard manager");

            return Ok(());
        }
    };

    let mut manager = shard_manager.lock();

    msg.reply(&ctx, "Shutting down!")?;

    // TODO reenable sentry
    // if let Some(client) = Hub::current().client() {
    //     client.close(Some(Duration::from_secs(2)));
    // }

    manager.shutdown_all();
    Ok(())
}
