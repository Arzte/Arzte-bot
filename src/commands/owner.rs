use core::structs::ShardManagerContainer;

command!(quit(ctx, msg, _args) {
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
use serenity::Result;
use std::process::Command;
use std::thread;

command!(update(ctx, msg, _args) {
    let github_json: Github = reqwest::get("https://api.github.com/repos/Arzte/Arzte-bot/commits/master")?.json()?;
    let github_latest_sha = github_json.sha;
    let github_short = &github_latest_sha[0..7];

    if let Some(local_short) = built_info::GIT_VERSION {
        if local_short == github_short {
            msg.channel_id.say(format!("Already at latest version!\n{}", github_json.html_url))?;
            return Ok(())
        }
    };
    let ctx = ctx.clone();
    let msg = msg.clone();

    thread::spawn(move || -> Result<()> {
            if let Ok(mut msg_status) = msg.channel_id.say("Now updating Arzte's Cute Bot, please wait....") {

                    msg_status.edit(|m| m.content("Pulling in the latest changes from github...."))?;

                    let output = Command::new("git")
                        .args(&["pull", "-ff"])
                        .output()?;

                    if output.status.success() {
                        msg_status.edit(|m| m.content("Finished pulling updates from Github."))?;
                    } else {
                        msg_status.edit(|m| m.content("Failed to pull updates from Github."))?;
                        msg.channel_id.say(format!("**```\n{}\n```**", String::from_utf8_lossy(&output.stdout)))?;
                        msg.channel_id.say("Update failed! :(")?;
                        return Ok(())
                    }

                    msg_status.edit(|m| m.content("Now compiling changes.... (This takes a long time)"))?;
                    msg.channel_id.broadcast_typing()?;
                    let output2 = Command::new("/home/faey/.cargo/bin/cargo")
                        .args(&["+stable", "build", "--release"])
                        .current_dir("/home/faey/bot")
                        .output()?;

                    if output2.status.success() {
                        msg_status.edit(|m| m.content("Finished compiling new changes."))?;
                    } else {
                        msg_status.edit(|m| m.content("Failure while compiling new changes."))?;
                        msg.channel_id.say(format!("**```\n{}\n```**", String::from_utf8_lossy(&output2.stderr)))?;
                        msg.channel_id.say("Update failed! :(")?;
                        return Ok(())
                    }

            }

            if let Ok(mut msg_shard) = msg.channel_id.say("Getting shard manager, then telling the bot to shutdown...") {
                let data = ctx.data.lock();

                let shard_manager = match data.get::<ShardManagerContainer>() {
                    Some(v) => v,
                    None => {
                        let _ = msg_shard.edit(|m| m.content("There was a problem getting the shard manager"));

                        return Ok(())
                    },
                };

                let mut manager = shard_manager.lock();

                msg_shard.edit(|m| m.content("Updated! Restarting now!"))?;

                manager.shutdown_all();
            }
            Ok(())
    });
});
