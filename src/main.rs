#[macro_use] extern crate log;
#[macro_use] extern crate serenity;
extern crate config;
extern crate chrono;
extern crate env_logger;
extern crate kankyo;
extern crate rand;
extern crate typemap;

mod commands;

use serenity::client::bridge::gateway::ShardManager;
use serenity::framework::StandardFramework;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::http;
use std::collections::HashSet;
use serenity::prelude::Mutex;
use std::sync::Arc;
use typemap::Key;
use std::env;
use env_logger::{Builder, Target};

pub struct ShardManagerContainer;

impl Key for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

fn main() {
    let mut builder = Builder::new();
    builder.target(Target::Stdout);
    if env::var("LOG").is_ok() {
        builder.parse(&env::var("LOG").unwrap());
    }
    builder.init();

    // Settings file, used currently for token
    let mut settings = config::Config::default();
    settings.merge(config::File::with_name("Settings")).expect("No file called Settings.toml in same folder as bot");
    // set settings to variables that can be accessed.
    let token = settings.get_str("token").expect("No token/token value set in Settings file");

    let mut client = Client::new(&token, Handler).expect("Err creating client");

    {
        let mut data = client.data.lock();
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    let owners = match http::get_current_application_info() {
        Ok(info) => {
            let mut set = HashSet::new();
            set.insert(info.owner.id);

            set
        },
        Err(why) => panic!("Couldn't get application info: {:?}", why),
    };

    client.with_framework(StandardFramework::new()
        .configure(|c| c
            .owners(owners)
            .prefix(".db "))
        .command("ping", |c| c.cmd(commands::meta::ping))
        .command("multiply", |c| c.cmd(commands::math::multiply))
        .command("quit", |c| c
            .cmd(commands::owner::quit)
            .owners_only(true)));

    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
    }
}
