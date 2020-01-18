use crate::ShardManagerContainer;
use chrono::Duration;
use log::{
    error,
    info,
    trace,
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
    structs::{
        GithubCommit,
        GithubRelease,
        SettingsContainer,
    },
    utils::dn_file,
};

#[command]
fn update(ctx: &mut Context, msg: &Message) -> CommandResult {
    let github_commit_json: GithubCommit =
        reqwest::get("https://api.github.com/repos/Arzte/Arzte-bot/commits/master")?.json()?;
    let github_release_json: GithubRelease =
        reqwest::get("https://api.github.com/repos/Arzte/Arzte-bot/releases/latest")?.json()?;
    let github_commit_sha = &github_commit_json.sha[0..7];
    let github_release_tag = github_release_json.tag_name.as_str();

    let debug = {
        let data = ctx.data.read();

        trace!("Getting settings mutex from data...");
        let settings_manager = {
            match data.get::<SettingsContainer>() {
                Some(v) => v,
                None => {
                    error!("Error getting settings container.");

                    return Ok(());
                }
            }
        };

        trace!("Getting lock for settings manager");
        let settings = settings_manager.try_lock()?;
        settings.get_bool("debug").unwrap_or(false)
    };

    if let (false, Some(local_git)) = (debug, built_info::GIT_VERSION) {
        let num_local: i32 = local_git.replace(".", "").parse::<i32>()?;
        let num_github: i32 = github_release_tag.replace(".", "").parse::<i32>()?;

        if local_git == github_commit_sha
            || num_local > num_github
            || local_git == github_release_tag
        {
            if let Ok(msg_latest) = msg.channel_id.say(&ctx.http, "Already at latest version!") {
                std::thread::sleep(std::time::Duration::from_secs(3));
                let _latest_delete_msg = msg_latest.delete(&ctx);
                if let Err(_missing_perms) = msg.delete(&ctx) {}
            }
            return Ok(());
        } else if github_release_json.assets.is_empty() {
            if let Ok(msg_latest) = msg.channel_id.say(&ctx.http, "There's a release, however Travis hasn't successfully built the new version yet, perhaps try again in a few minutes?") {
                std::thread::sleep(std::time::Duration::from_secs(10));
                let _ = msg_latest.delete(&ctx);
                let _ = msg.delete(&ctx);
            }
        }
    };
    let mut message = msg
        .channel_id
        .say(&ctx.http, "Now updating Arzte's Cute Bot, please wait....")?;

    let github_release_download = &github_release_json.assets[0].browser_download_url;
    let github_release_download = github_release_download.clone();

    trace!("Downloading the latest release from github...");
    dn_file(&github_release_download, "arzte.tar.gz", "arzte")?;

    info!("Telling raven to finish what it is doing");
    if let Some(client) = Hub::current().client() {
        client.close(Some(Duration::seconds(2).to_std()?));
    }

    trace!("Getting serenity's data lock...");
    let data = ctx.data.write();

    message.edit(&ctx, |m| m.content("Updated! Restarting now!"))?;

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            error!("Couldn't get the shard manager for a graceful shutdown, killing the bot....");
            std::process::exit(0);
        }
    };

    trace!("Getting a lock on shard_manager");
    if let Some(mut manager) = shard_manager.try_lock() {
        info!("Telling serenity to close all shards, then shutdown");
        manager.shutdown_all();
    } else {
        error!("Couldn't get the shard manager lock for a graceful shutdown, killing the bot...");
        std::process::exit(0);
    }

    Ok(())
}
