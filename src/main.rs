#[macro_use]
extern crate log;
#[macro_use]
extern crate serenity;
extern crate chrono;
extern crate config;
extern crate env_logger;
extern crate kankyo;
extern crate rand;
extern crate typemap;

mod commands;

use env_logger::{Builder, Target};
use serenity::client::bridge::gateway::ShardManager;
use serenity::framework::standard::{
    help_commands, DispatchError, HelpBehaviour, StandardFramework,
};
use serenity::http;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::prelude::Mutex;
use serenity::prelude::*;
use std::collections::HashSet;
use std::env;
use std::sync::Arc;
use typemap::Key;

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
    settings
        .merge(config::File::with_name("Settings"))
        .expect("No file called Settings.toml in same folder as bot");
    // set settings to variables that can be accessed.
    let token = settings
        .get_str("token")
        .expect("No token/token value set in Settings file");

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
        }
        Err(why) => panic!("Couldn't get application info: {:?}", why),
    };

    client.with_framework(
        StandardFramework::new()
            .configure(|c| {
                c.owners(owners)
                    .allow_whitespace(true)
                    .on_mention(true)
                    .prefix(".db")
                    .no_dm_prefix(true)
                    .case_insensitivity(true)
                    .prefix_only_cmd(commands::info::about)
            }).after(|_ctx, msg, cmd_name, error| {
                //  Print out an error if it happened
                if let Err(why) = error {
                    if let Err(why) = msg.channel_id.say("Unexpected error when exacuting command, please try again later.") {
                        error!("Error sending message: {}", why);
                    };
                    error!("Error in {}: {:?}", cmd_name, why);
                }
            })
            .on_dispatch_error(|_ctx, msg, error| {
                if let DispatchError::RateLimited(seconds) = error {
                    let _ = msg
                        .channel_id
                        .say(&format!("Try this again in {} seconds.", seconds));
                }
            }).customised_help(help_commands::with_embeds, |c| {
                c.individual_command_tip("If you want more information about a specific command, just pass the command as argument.")
                .command_not_found_text("Could not find: `{}`.")
                // Define the maximum Levenshtein-distance between a searched command-name
                // and commands. If the distance is lower than or equal the set distance,
                // it will be displayed as a suggestion.
                // Setting the distance to 0 will disable suggestions.
                .max_levenshtein_distance(3)
                // If a user lacks permissions for a command, hide the command.
                .lacking_permissions(HelpBehaviour::Hide)
                // If the user is nothing but lacking a certain role, display it.
                .lacking_role(HelpBehaviour::Nothing)
                // The last `enum`-variant is `Strike`, which ~~strikes~~ a command.
                .wrong_channel(HelpBehaviour::Strike)
            }).command("about", |c| c.cmd(commands::info::about))
            .group("Ultility", |g| g
                .command("ping", |c| c.cmd(commands::meta::ping))
                .command("multiply", |c| c.cmd(commands::math::multiply)))
            .group("Bot Owner Only", |g| g
                .command("quit", |c| c.cmd(commands::owner::quit).owners_only(true))),
    );

    if let Err(why) = client.start_autosharded() {
        error!("Client error: {:?}", why);
    }
}
