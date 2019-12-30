#![recursion_limit = "128"]
#![allow(proc_macro_derive_resolution_fallback)]

mod commands;
pub mod core;

use log::{
    error,
    info,
};
use serenity::{
    framework::{
        standard::macros::group,
        StandardFramework,
    },
    model::{
        event::ResumedEvent,
        gateway::Ready,
    },
    prelude::{
        Client,
        Context,
        EventHandler,
    },
};
use std::{
    collections::HashSet,
    env,
    sync::Arc,
};

use crate::commands::info::*;
use crate::commands::math::*;
use crate::commands::owner::*;
use crate::core::structs::{
    SettingsContainer,
    ShardManagerContainer,
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

// TODO: Move owner commands to quit so not anyone can run them
group!({
    name: "general",
    options: {},
    commands: [about, guild, ping, quit, math]
});

fn main() {
    let _guard = sentry::init((
        "https://c667c4bf6a704b0f802fa075c98f8c03@sentry.io/1340627",
        sentry::ClientOptions {
            max_breadcrumbs: 50,
            environment: Some("staging".into()),
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

    // This will load the environment variables located at `./.env`, relative to
    // the CWD. See `./.env.example` for an example on how to structure this.
    kankyo::load(false).expect("Failed to load .env file");

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let mut client = Client::new(&token, Handler).expect("Err creating client");

    {
        let mut data = client.data.write();
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
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
            .configure(|c| c.owners(owners).prefix("~"))
            .group(&GENERAL_GROUP),
    );

    if let Err(why) = client.start_autosharded() {
        error!("Client error: {:?}", why);
    }
}
