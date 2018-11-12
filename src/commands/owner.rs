use super::super::ShardManagerContainer;

command!(quit(ctx, msg, _args) {
    // The shard manager is an interface for mutating, stopping, restarting, and
    // retrieving information about shards.
    let data = ctx.data.lock();

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            let _ = msg.reply("There was a problem getting the shard manager");

            return Ok(());
        },
    };

    let mut manager = shard_manager.lock();

    msg.reply("Shutting down!")?;

    manager.shutdown_all();
});

command!(update(ctx, msg, _args) {
    msg.channel_id.broadcast_typing()?;
    if let Ok(mut message) = msg.channel_id.say("Now updating Arzte's Cute Bot, please wait....") {
        // The shard manager is an interface for mutating, stopping, restarting, and
        // retrieving information about shards.
        let data = ctx.data.lock();

        let shard_manager = match data.get::<ShardManagerContainer>() {
            Some(v) => v,
            None => {
                let _ = msg.reply("There was a problem getting the shard manager");

                return Ok(());
            },
        };

        let mut manager = shard_manager.lock();

        message.edit(|m| m.content("Updated! Restarting now!"))?;

        manager.shutdown_all();
    };
});
