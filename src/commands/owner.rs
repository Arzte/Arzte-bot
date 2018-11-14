use core::structs::ShardManagerContainer;

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

use std::process::Command;

command!(update(ctx, msg, _args) {
    msg.channel_id.broadcast_typing()?;
    if let Ok(mut message) = msg.channel_id.say("Now updating Arzte's Cute Bot, please wait....") {

        if let Ok(mut cmd_output) = msg.channel_id.say("**```\n \n```**") {
            message.edit(|m| m.content("Pulling in the latest changes from github...."))?;

            let output = Command::new("git")
                .args(&["pull", "-ff"])
                .output()?;

            cmd_output.edit(|m| m.content(format!("**```\n{}\n```**", String::from_utf8_lossy(&output.stdout))))?;
            message.edit(|m| m.content("Finished pulling updates from Github."))?;

            std::thread::sleep(std::time::Duration::from_millis(1000));

            message.edit(|m| m.content("Now compiling changes.... (This takes a long time)"))?;
            let output2 = Command::new("/home/faey/.cargo/bin/cargo")
                .args(&["+stable", "build", "--release"])
                .current_dir("/home/faey/bot")
                .output()?;

            cmd_output.edit(|m| m.content(format!("**```\n{}\n```**", String::from_utf8_lossy(&output2.stderr))))?;
            message.edit(|m| m.content("Finished compiling new changes."))?;
        }

    }

    if let Ok(mut shard) = msg.channel_id.say("Getting shard manager, then telling the bot to shutdown...") {
        // The shard manager is an interface for mutating, stopping, restarting, and
        // retrieving information about shards.
        let data = ctx.data.lock();

        let shard_manager = match data.get::<ShardManagerContainer>() {
            Some(v) => v,
            None => {
                let _ = shard.edit(|m| m.content("There was a problem getting the shard manager"));

                return Ok(());
            },
        };

        let mut manager = shard_manager.lock();

        shard.edit(|m| m.content("Updated! Restarting now!"))?;

        manager.shutdown_all();
    }
});
