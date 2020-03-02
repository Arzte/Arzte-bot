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
        Args,
        CommandResult,
    },
    model::prelude::Message,
    prelude::Context,
    utils::{
        content_safe,
        ContentSafeOptions,
    },
};
use std::{
    fs,
    fs::File,
    os::unix::fs::PermissionsExt,
    path::Path,
};

#[command]
#[aliases("s")]
// Repeats what the user passed as argument but ensures that user and role
// mentions are replaced with a safe textual alternative.
// In this example channel mentions are excluded via the `ContentSafeOptions`.
fn say(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let settings = if msg.guild_id.is_some() {
        // By default roles, users, and channel mentions are cleaned.
        ContentSafeOptions::default()
            // We do not want to clean channal mentions as they
            // do not ping users.
            .clean_channel(false)
            // We don't want to clean user mentions, as it was
            // probably intentionally pinging
            .clean_user(false)
    } else {
        ContentSafeOptions::default()
            .clean_channel(false)
            .clean_role(false)
            // We don't want to clean user mentions, as it was probably
            // intentionally pinging
            .clean_user(false)
    };

    let content = content_safe(&ctx.cache, &args.rest(), &settings);

    if let Err(why) = msg.channel_id.say(&ctx.http, &content) {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}

#[command]
#[aliases("q")]
/// Kills this instance of the bot, only available to bot owners
fn quit(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.reply(&ctx, "Shutting down!")?;

    if let Some(client) = Hub::current().client() {
        client.close(Some(Duration::seconds(2).to_std()?));
    }
    {
        let shard_manager = {
            let data = match ctx.data.try_read() {
                Some(v) => v,
                None => {
                    error!("Couldn't get data lock for a graceful shutdown, killing bot");
                    std::process::exit(0)
                }
            };

            match data.get::<ShardManagerContainer>() {
                Some(v) => std::sync::Arc::clone(v),
                None => {
                    error!(
                        "Couldn't get the shard manager for a graceful shutdown, killing the bot...."
                    );
                    std::process::exit(0)
                }
            }
        };

        if let Some(mut manager) = shard_manager.try_lock() {
            info!("Telling serenity to close all shards, then shutdown");
            manager.shutdown_all();
        } else {
            error!(
                "Couldn't get the shard manager lock for a graceful shutdown, killing the bot..."
            );
            std::process::exit(0)
        };
    }

    Ok(())
}

use crate::core::{
    built_info,
    structs::GithubRelease,
    structs::GithubTag,
};

#[command]
#[aliases("up")]
/// Downloads the latest version of the bot if available, only available to bot owners
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

    if bot_verison == github_latest_tag_verison {
        if let Ok(msg_latest) = msg.channel_id.say(&ctx.http, "Already at latest version!") {
            std::thread::sleep(std::time::Duration::from_secs(10));
            // If the message can't be deleted, don't delete at all
            if !msg.delete(&ctx).is_err() {
                msg_latest.delete(&ctx)?;
            }
        }
        return Ok(());
    } else if github_latest_release.assets.is_empty()
        || github_latest_tag_verison > github_latest_release_version
    {
        if let Ok(msg_latest) = msg.channel_id.say(&ctx.http, "There's a release, however Travis hasn't successfully built the new version yet, perhaps try again in a few minutes?") {
                std::thread::sleep(std::time::Duration::from_secs(10));
                // If the message can't be deleted, don't delete at all
                if !msg.delete(&ctx).is_err() {
                    msg_latest.delete(&ctx)?;
                }
            }
        return Ok(());
    }

    let mut message = msg.channel_id.say(
        &ctx.http,
        "Now downloading a new version of Arzte's Cute Bot, please wait....",
    )?;

    trace!("Downloading the latest release from github...");

    let download_file = "arzte.tar.gz";
    let mut response = reqwest
        .get(&github_latest_release.assets[0].browser_download_url)
        .send()?;
    let mut download = File::create(download_file)?;
    let dest = Path::new(download_file);

    response.copy_to(&mut download)?;

    message.edit(&ctx, |m| {
        m.content("Download complete, extracting new version from downloaded archive.....")
    })?;
    trace!("Extracting from downloaded archive");
    let tar_gz = File::open(dest)?;
    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut ar = tar::Archive::new(tar);
    ar.unpack(".")?;

    trace!("Getting and computing the bin hashes");
    let bin_path = Path::new("arzte-bot");
    let hash = blake2b_simd::blake2b(&fs::read(bin_path)?[..]).to_hex();
    let bin_hash_path = Path::new("arzte-bot.blake2");
    let bin_hash = fs::read_to_string(bin_hash_path)?;
    let bin_hash_str = bin_hash.trim_end_matches('\n');
    fs::remove_file(bin_hash_path)?;

    if &hash != bin_hash_str {
        message.edit(&ctx, |m| {
            m.content("Hash check failed, can't update Arzte's Cute Bot")
        })?;
        fs::remove_file(bin_path)?;
        return Ok(());
    }

    message.edit(&ctx, |m| {
        m.content("Download was successful, updating Arzte's Cute Bot....")
    })?;
    fs::rename(bin_path, "arzte")?;
    fs::metadata("arzte")?.permissions().set_mode(0o755);

    // Leave archive till last in case of a failure before this point (easier to fix ssh wise)
    fs::remove_file(dest)?;

    info!("Telling raven to finish what it is doing");
    if let Some(client) = Hub::current().client() {
        client.close(Some(Duration::seconds(2).to_std()?));
    }

    message.edit(&ctx, |m| m.content("Updated! Restarting now!"))?;

    {
        let shard_manager = {
            trace!("Getting serenity's data lock...");
            let data = match ctx.data.try_read() {
                Some(data_lock) => data_lock,
                None => {
                    error!("Couldn't get data lock for a graceful shutdown, killing the bot...");
                    std::process::exit(0);
                }
            };
            match data.get::<ShardManagerContainer>() {
                Some(v) => std::sync::Arc::clone(v),
                None => {
                    error!(
                        "Couldn't get the shard manager for a graceful shutdown, killing the bot...."
                    );
                    std::process::exit(0);
                }
            }
        };

        trace!("Getting a lock on shard_manager");
        if let Some(mut manager) = shard_manager.try_lock() {
            info!("Telling serenity to close all shards, then shutdown");
            manager.shutdown_all();
        } else {
            error!(
                "Couldn't get the shard manager lock for a graceful shutdown, killing the bot..."
            );
            std::process::exit(0);
        };
    }

    Ok(())
}
