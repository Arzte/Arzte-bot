#![recursion_limit = "128"]
#![allow(proc_macro_derive_resolution_fallback)]
#![feature(try_blocks)]

mod commands;
pub mod core;

#[allow(unused_imports)]
use log::{
    error,
    info,
    trace,
    warn,
};

use serenity::{
    framework::{
        standard::{
            help_commands,
            macros::{
                group,
                help,
            },
            Args,
            CommandGroup,
            CommandResult,
            DispatchError,
            HelpOptions,
        },
        StandardFramework,
    },
    model::{
        event::ResumedEvent,
        gateway::Ready,
        prelude::*,
    },
    prelude::{
        Client,
        Context,
        EventHandler,
    },
};
use std::{
    collections::HashSet,
    sync::Arc,
    sync::Mutex,
};

use crate::{
    commands::{
        info::*,
        math::*,
        owner::*,
    },
    core::structs::{
        SettingsContainer,
        ShardManagerContainer,
    },
};

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

#[group]
#[commands(about, user, avatar, guild, ping, math)]
struct General;

#[group]
#[owners_only]
#[description = "Commands that can only be ran by the owner of the bot"]
#[commands(quit, update)]
struct Owners;

#[help]
#[lacking_permissions = "Hide"]
#[wrong_channel = "Strike"]
fn my_help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, &help_options, groups, owners)
}

fn main() {
    let config = Arc::new(Mutex::new(config::Config::default()));

    let (token, enviroment) = {
        let mut settings = config.lock().unwrap_or_else(|err| {
            error!("Unable to get config lock, bailing...");
            panic!("{}", err);
        });

        settings
            .set_default("debug", false)
            .expect("Unable to set a default value for debug");
        settings
            .merge(config::File::with_name("settings"))
            .expect("No file called Settings.toml in same folder as bot");

        let token = {
            settings
                .get_str("token")
                .expect("No token/token value set in Settings file")
        };
        let enviroment = {
            if !settings.get_bool("debug").unwrap_or_else(|_err| false) {
                "Production"
            } else {
                "Staging"
            }
        };

        (token, enviroment)
    };

    let _guard = sentry::init((
        "https://c667c4bf6a704b0f802fa075c98f8c03@sentry.io/1340627",
        sentry::ClientOptions {
            max_breadcrumbs: 50,
            environment: Some(enviroment.into()),
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));

    let mut log_builder = pretty_env_logger::formatted_builder();
    log_builder.parse_filters("info");
    sentry::configure_scope(|scope| {
        scope.set_level(Some(sentry::Level::Warning));
    });
    sentry::integrations::env_logger::init(Some(log_builder.build()), Default::default());
    sentry::integrations::panic::register_panic_handler();

    let mut client = Client::new(&token, Handler).expect("Err creating client");

    {
        let mut data = client.data.write();
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<SettingsContainer>(Arc::clone(&config));
    }

    let owners = match client.cache_and_http.http.get_current_application_info() {
        Ok(info) => {
            let mut set = HashSet::new();
            set.insert(info.owner.id);

            set
        }
        Err(why) => panic!("Couldn't get application info: {:?}", why),
    };

    client.with_framework(
        StandardFramework::new()
            .configure(|c| {
                c.owners(owners)
                // TODO: Make this prefix dynamic and configurable per guild
                    .prefix("a.")
                    .ignore_webhooks(false)
                    .case_insensitivity(true)
            })
            .on_dispatch_error(|context, message, error| match error {
                DispatchError::Ratelimited(seconds) => {
                    let _ = message.channel_id.say(
                        &context.http,
                        &format!("Try this again in {} seconds.", seconds),
                    );
                },
                DispatchError::OnlyForOwners => {},
                _ => error!("{} failed: {:?}", message.content, error),
            })
            .after(|context, message, command_name, error| if let Err(why) = error {
                    let _ = message.channel_id.say(&context.http, "An unexpected error occured when running this command, please try again later.");
                    error!("Command {} triggered by {}: {:#?}", command_name, message.author.tag(), why);
            })
            .help(&MY_HELP)
            .group(&GENERAL_GROUP)
            .group(&OWNERS_GROUP)
    );

    if let Err(why) = client.start_autosharded() {
        error!("Client error: {:?}", why);
    }
}
