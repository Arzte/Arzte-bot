use crate::ShardManagerContainer;
use chrono::Duration;
#[allow(unused_imports)]
use log::{
    error,
    info,
    trace,
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
use std::{
    fs,
    os::unix::fs::PermissionsExt,
};

#[command]
fn quit(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.reply(&ctx, "Shutting down!")?;

    if let Some(client) = Hub::current().client() {
        client.close(Some(Duration::seconds(2).to_std()?));
    }

    let data = ctx.data.write();

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            error!("Couldn't get the shard manager for a graceful shutdown, killing the bot....");
            std::process::exit(0)
        }
    };

    if let Some(mut manager) = shard_manager.try_lock() {
        info!("Telling serenity to close all shards, then shutdown");
        manager.shutdown_all();
    } else {
        error!("Couldn't get the shard manager lock for a graceful shutdown, killing the bot...");
        std::process::exit(0)
    }

    Ok(())
}

use crate::core::{
    built_info,
    structs::GithubRelease,
    structs::GithubTag,
};

#[command]
fn update(ctx: &mut Context, msg: &Message) -> CommandResult {
    let reqwest = reqwest::blocking::ClientBuilder::new()
        .user_agent(format!(
            "{}/{}",
            built_info::PKG_NAME,
            built_info::PKG_VERSION
        ))
        .build()?;
    let github_latest_release: GithubRelease = reqwest
        .get("https://api.github.com/repos/Arzte/Arzte-bot/releases/latest")
        .send()?
        .json()?;
    let github_tags: GithubTag = reqwest
        .get("https://api.github.com/repos/Arzte/Arzte-bot/tags")
        .send()?
        .json()?;
    let github_latest_release_tag = github_latest_release.tag_name.as_str();
    let bot_verison = semver::Version::parse(built_info::PKG_VERSION)?;
    let github_latest_release_version = semver::Version::parse(github_latest_release_tag)?;
    let github_latest_tag_verison = semver::Version::parse(github_tags[0].name.as_ref())?;

    if bot_verison == github_latest_release_version {
        if let Ok(msg_latest) = msg.channel_id.say(&ctx.http, "Already at latest version!") {
            std::thread::sleep(std::time::Duration::from_secs(10));
            // If the message can't be deleted, don't delete at all
            if let Err(_) = msg.delete(&ctx) {
                let _ = msg_latest.delete(&ctx);
            };
        }
        return Ok(());
    } else if github_latest_release.assets.is_empty()
        || github_latest_tag_verison > github_latest_release_version
    {
        if let Ok(msg_latest) = msg.channel_id.say(&ctx.http, "There's a release, however Travis hasn't successfully built the new version yet, perhaps try again in a few minutes?") {
                std::thread::sleep(std::time::Duration::from_secs(10));
                // If the message can't be deleted, don't delete at all
                if let Err(_) = msg.delete(&ctx) {
                    let _ = msg_latest.delete(&ctx);
                };
            }
        return Ok(());
    }

    let mut message = msg
        .channel_id
        .say(&ctx.http, "Now updating Arzte's Cute Bot, please wait....")?;

    trace!("Downloading the latest release from github...");

    let download_file = "arzte.tar.gz";
    let mut response = reqwest
        .get(&github_latest_release.assets[0].browser_download_url)
        .send()?;
    let mut download = std::fs::File::create(download_file)?;
    let dest = std::path::Path::new(download_file);

    response.copy_to(&mut download)?;

    trace!("Opening the file.");
    let tar_gz = fs::File::open(dest)?;
    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut ar = tar::Archive::new(tar);
    ar.unpack(".")?;

    fs::remove_file(dest)?;

    let bin = std::path::Path::new("arzte");
    fs::metadata(bin)?.permissions().set_mode(0o755);

    info!("Telling raven to finish what it is doing");
    if let Some(client) = Hub::current().client() {
        client.close(Some(Duration::seconds(2).to_std()?));
    }

    trace!("Getting serenity's data lock...");
    let data = ctx.data.write();

    if let Err(err) = message
        .edit(&ctx, |m| m.content("Updated! Restarting now!"))
        .and_then(|_t| {
            let shard_manager = match data.get::<ShardManagerContainer>() {
                Some(v) => v,
                None => {
                    error!(
                    "Couldn't get the shard manager for a graceful shutdown, killing the bot...."
                );
                    std::process::exit(0);
                }
            };

            trace!("Getting a lock on shard_manager");
            if let Some(mut manager) = shard_manager.try_lock() {
                info!("Telling serenity to close all shards, then shutdown");
                manager.shutdown_all();
                Ok(())
            } else {
                error!(
                "Couldn't get the shard manager lock for a graceful shutdown, killing the bot..."
            );
                std::process::exit(0);
            }
        })
    {
        error!(
            "Couldn't edit message: {:?}\n ungracefully killing bot",
            err
        );
        std::process::exit(0);
    }

    Ok(())
}
