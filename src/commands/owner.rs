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
    io,
    os::unix::fs::PermissionsExt,
};
use tempdir::TempDir;

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
        GithubRelease,
        SettingsContainer,
    },
};

#[command]
fn update(ctx: &mut Context, msg: &Message) -> CommandResult {
    let github_release_json: GithubRelease =
        reqwest::get("https://api.github.com/repos/Arzte/Arzte-bot/releases/latest")?.json()?;
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

    if let (false, pkg_version) = (debug, built_info::PKG_VERSION) {
        let local_verison = semver::Version::parse(pkg_version)?;
        let github_verison = semver::Version::parse(github_release_tag)?;

        if local_verison == github_verison {
            if let Ok(msg_latest) = msg.channel_id.say(&ctx.http, "Already at latest version!") {
                std::thread::sleep(std::time::Duration::from_secs(3));
                let _latest_delete_msg = msg_latest.delete(&ctx);
                let _missing_perms = msg.delete(&ctx);
            }
            return Ok(());
        } else if github_release_json.assets.is_empty() && local_verison > github_verison {
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

    trace!("Downloading the latest release from github...");
    // In a seperate context because TempDir only needs to exist till the file is finished downloading
    {
        let tmp_dir = TempDir::new("arzte.download")?;
        let download_file = "arzte.tar.gz";
        let final_file = "arzte";
        let mut response = reqwest::get(&github_release_json.assets[0].browser_download_url)?;

        let mut dest = fs::File::create(tmp_dir.path().join(download_file))?;

        io::copy(&mut response, &mut dest)?;

        trace!("Opening the file.");
        let tar_gz = fs::File::open(tmp_dir.path().join(&download_file))?;
        let tar = flate2::read::GzDecoder::new(tar_gz);
        let mut ar = tar::Archive::new(tar);
        ar.unpack(".")?;

        let file = format!("{}/{}", ".", final_file);
        let dest = std::path::Path::new(&file);

        trace!("Copying bot bin to replace old bot bin");
        fs::copy(tmp_dir.path().join(final_file), dest)?;

        fs::metadata(dest)?.permissions().set_mode(0o775);
    }

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
