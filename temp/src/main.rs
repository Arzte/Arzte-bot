extern crate arzte;
extern crate serenity;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate sentry;

use serenity::{http, 
    prelude::*, 
    model::event::ResumedEvent, 
    model::gateway::Ready, 
    model::id::ChannelId, 
    framework::standard::{
        help_commands, DispatchError, HelpBehaviour, StandardFramework,
    }};
use arzte::{commands::*, core::structs::{ShardManagerContainer, SettingsContainer}};
use env_logger::{Builder, Target};
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
    // Sentry error stuffs
    let _guard = sentry::init(("https://c667c4bf6a704b0f802fa075c98f8c03@sentry.io/1340627", sentry::ClientOptions {
        max_breadcrumbs: 50,
        debug: true,
        environment: Some("staging".into()),
        ..Default::default()
    }));
    
    // env_logger setup stuffs
    let mut builder = Builder::new();
    builder.target(Target::Stdout);
    if env::var("LOG").is_ok() {
        builder.parse(&env::var("LOG").unwrap());
    }

    sentry::configure_scope(|scope| {
        scope.set_level(Some(sentry::Level::Warning));
    });
    sentry::integrations::env_logger::init(Some(builder.build()), Default::default());
    sentry::integrations::panic::register_panic_handler();


    let config = Arc::new(Mutex::new(config::Config::default()));

    let token = {
        let mut settings = config.lock();
        settings.set_default("debug", "false")
            .map_err(|err| warn!("Error setting default debug value: {}", err)).expect("error mapping error, lmao.");
        settings
            .merge(config::File::with_name("settings"))
            .expect("No file called Settings.toml in same folder as bot");
        settings
            .get_str("token")
            .expect("No token/token value set in Settings file")
    };

    let mut client = Client::new(&token, Handler).expect("Err creating client");

    {
        let mut data = client.data.lock();
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<SettingsContainer>(Arc::clone(&config));
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
                    if let Err(msg_why) = msg.channel_id.say("An unexpected error occured when running this command, please try again later.") {
                        error!("Error sending message: {:#?}", msg_why);
                    };
                    if let Err(msg_why) = ChannelId(521_537_902_291_976_196).send_message(|m| m.content(format!("An unaccounted for error occured in ``{}``, details on Sentry.", cmd_name))) {
                        error!("Error sending error message: {:#?}", msg_why);
                    };
                    error!("{} has encountered an error:: {:?}", cmd_name, why);
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
