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

use core::built_info;
use core::structs::Github;
use std::process::Command;

command!(update(ctx, msg, _args) {
    msg.channel_id.broadcast_typing()?;

    let github_json: Github = reqwest::get("https://api.github.com/repos/Arzte/Arzte-bot/commits/master")?.json()?;
    let github_latest_sha = github_json.sha;
    let github_short = &github_latest_sha[0..7];

    if let Some(local_short) = built_info::GIT_VERSION {
        if local_short != github_short {
            msg.channel_id.say(format!("There is a update available! :)\nHowever for testing I'm only outputting the values of the short commit I was built on, and GitHub. :(\nlocal: ``{}`` matches github: ``{}``", local_short, github_short))?;
            return Ok(())
        } else {
            msg.channel_id.say(format!("There are no updates available, perhaps you forgot to push to Github?\nlocal: ``{}`` matchs github:``{}``", local_short, github_short))?;
            return Ok(())
        }
    };

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
