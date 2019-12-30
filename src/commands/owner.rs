use crate::ShardManagerContainer;
use chrono::Duration;
use log::{
    debug,
    error,
    info,
    warn,
};
use sentry::Hub;
use serenity::{
    framework::standard::{
        macros::command,
        CommandResult,
    },
    model::prelude::Message,
    prelude::Context,
};

#[command]
#[owners_only]
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

    if let Some(client) = Hub::current().client() {
        client.close(Some(Duration::seconds(2).to_std()?));
    }

    info!("Telling serenity to close all shards, then shutdown");
    manager.shutdown_all();
    Ok(())
}

use crate::core::{
    built_info,
    structs::{
        GithubCommit,
        GithubRelease,
        SettingsContainer,
    },
    utils::dn_file,
};
use std::{
    process::Command,
    thread,
};

// TODO: refactor to simplify
#[command]
#[owners_only]
fn update(ctx: &mut Context, msg: &Message) -> CommandResult {
    let github_commit_json: GithubCommit =
        reqwest::get("https://api.github.com/repos/Arzte/Arzte-bot/commits/master")?.json()?;
    let github_release_json: GithubRelease =
        reqwest::get("https://api.github.com/repos/Arzte/Arzte-bot/releases/latest")?.json()?;
    let github_commit_sha = github_commit_json.sha;
    let github_release_tag = github_release_json.tag_name.as_str();
    let github_short = &github_commit_sha[0..7];

    debug!("Getting debug state...");
    let debug = {
        debug!("Getting serenity's data container lock...");
        let data = ctx.data.write();

        debug!("Getting settings mutex from data...");
        let settings_manager = {
            match data.get::<SettingsContainer>() {
                Some(v) => v,
                None => {
                    error!("Error getting settings container.");

                    return Ok(());
                }
            }
        };

        debug!("Getting lock for settings manager");
        let settings = settings_manager.lock();
        settings.get_bool("debug")?
    };

    debug!("checking debug mode");
    if let (false, Some(local_git)) = (debug, built_info::GIT_VERSION) {
        let num_local: i32 = local_git
            .replace(".", "")
            .replace("-alpha", "")
            .parse::<i32>()?;
        let num_github: i32 = github_release_tag
            .replace(".", "")
            .replace("-alpha", "")
            .parse::<i32>()?;

        if local_git == github_short || num_local > num_github || local_git == github_release_tag {
            if let Ok(msg_latest) = msg.channel_id.say(&ctx.http, "Already at latest version!") {
                std::thread::sleep(std::time::Duration::from_secs(3));
                let _latest_delete_msg = msg_latest.delete(&ctx);
                if let Err(_missing_perms) = msg.delete(&ctx) {}
            }
            return Ok(());
        } else if github_release_json.assets.is_empty() {
            if let Ok(msg_latest) = msg.channel_id.say(&ctx.http, "There's a release, however Travis hasn't successfully built the new version yet, perhaps try again in a few minutes?") {
                std::thread::sleep(std::time::Duration::from_secs(10));
                let _latest_delete_msg = msg_latest.delete(&ctx);
                if let Err(_missing_perms) = msg.delete(&ctx) {}
            }
        }
    };

    let github_release_download = &github_release_json.assets[0].browser_download_url;
    let github_release_download = github_release_download.clone();
    let ctx = ctx.clone();
    let msg = msg.clone();

    thread::spawn(move || -> CommandResult {
        if let Ok(mut message) = msg
            .channel_id
            .say(&ctx.http, "Now updating Arzte's Cute Bot, please wait....")
        {
            debug!("Pulling in the latest changes from github....");

            if !debug {
                let output = Command::new("git").args(&["pull", "--rebase"]).output()?;

                if output.status.success() {
                    debug!("Finished pulling updates from Github.");
                } else {
                    error!(
                        "Failed to pull updates from Github:\n {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            }

            debug!("Downloading the latest release from github...");
            dn_file(&github_release_download, "arzte.tar.gz")?;
            debug!("Done downloading.");

            debug!("Opening the file.");
            let tar_gz = std::fs::File::open("arzte.tar.gz")?;
            debug!("Decompressing/Decoding arzte.tar.gz");
            let tar = flate2::read::GzDecoder::new(tar_gz);
            debug!("Telling tar the archive.");
            let mut ar = tar::Archive::new(tar);
            debug!("Unpacking tar archive");
            ar.unpack(".")?;

            debug!("Deleting leftover archive");
            std::fs::remove_file("arzte.tar.gz")?;

            debug!("Telling raven to finish what it is doing");
            if let Some(client) = Hub::current().client() {
                client.close(Some(Duration::seconds(2).to_std()?));
            }

            message.edit(&ctx, |m| m.content("Updated! Restarting now!"))?;

            debug!("Getting serenity's data lock...");
            let data = ctx.data.write();

            let shard_manager = match data.get::<ShardManagerContainer>() {
                Some(v) => v,
                None => {
                    warn!("Couldn't get the shard manager for a graceful shutdown, killing the bot....");
                    std::process::exit(0)
                }
            };

            debug!("Getting a lock on shard_manager");
            let mut manager = shard_manager.lock();

            info!("Telling serenity to close all shards, then shutdown");
            manager.shutdown_all();
        }
        Ok(())
    });
    Ok(())
}
