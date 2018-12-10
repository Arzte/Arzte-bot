extern crate arzte;
extern crate serenity;
#[macro_use]
extern crate log;
extern crate env_logger;

use serenity::model::id::ChannelId;
use arzte::commands::*;
use arzte::core::structs::ShardManagerContainer;
use env_logger::{Builder, Target};
use serenity::framework::standard::{
    help_commands, DispatchError, HelpBehaviour, StandardFramework,
};
use serenity::http;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::collections::HashSet;
use std::env;
use std::sync::Arc;

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

    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("Settings"))
        .expect("No file called Settings.toml in same folder as bot");
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
                    .prefix(".d")
                    .no_dm_prefix(true)
                    .case_insensitivity(true)
                    .prefix_only_cmd(info::about)
            }).after(|_ctx, msg, cmd_name, error| {
                //  Print out an error if it happened
                if let Err(why) = error {
                    if let Err(msg_why) = msg.channel_id.say("Unexpected error when exacuting command, please try again later.") {
                        error!("Error sending message: {:#?}", msg_why);
                    };
                    if let Err(msg_why) = ChannelId(521_537_902_291_976_196).send_message(|m| m.content(format!("An unaccounted for error occured!! pls fix: \n```rs\n{:#?}\n```", why))) {
                        error!("Error sending detail error message: {:#?}", msg_why);
                    };
                    error!("Error in {}: {:?}", cmd_name, why);
                }
            })
            .on_dispatch_error(|_ctx, msg, error| {
                // if there was an error related to ratelimiting, send a message about it.
                if let DispatchError::RateLimited(seconds) = error {
                    let _ = msg
                        .channel_id
                        .say(&format!("Try this again in {} seconds.", seconds));
                }
            }).customised_help(help_commands::with_embeds, |c| {
                c.individual_command_tip("If you want more information about a specific command, just pass the command as argument.")
                .command_not_found_text("Could not find: `{}`.")
                .max_levenshtein_distance(3)
                .lacking_permissions(HelpBehaviour::Hide)
                .lacking_role(HelpBehaviour::Nothing)
                .wrong_channel(HelpBehaviour::Strike)
            }).command("about", |c| c.cmd(info::about))
            .group("Ultility", |g| g
                .command("ping", |c| c.cmd(meta::ping))
                .command("guild", |c| c.cmd(info::guild))
                .command("math", |c| c.cmd(math::math)))
            .group("Bot Owner Only", |g| g
                .owners_only(true)
                .command("update", |c| c.cmd(owner::update).known_as("u"))
                .command("quit", |c| c.cmd(owner::quit))),
    );

    if let Err(why) = client.start_autosharded() {
        error!("Client error: {:?}", why);
    }
}
