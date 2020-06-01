#![recursion_limit = "128"]
#![allow(proc_macro_derive_resolution_fallback)]
#![feature(try_blocks, nll)]

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
    collections::{
        HashMap,
        HashSet,
    },
    sync::Arc,
    sync::Mutex,
};

use crate::{
    commands::{
        admin::*,
        info::*,
        math::*,
        owner::*,
    },
    core::{
        error::ReactionError,
        events::{
            reaction_add::reaction_add,
            reaction_remove::reaction_remove,
        },
        structs::{
            PoolContainer,
            PrefixHashMapContainer,
            SettingsContainer,
            ShardManagerContainer,
            TokioContainer,
        },
        utils::FancyPool,
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

    fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        if let Err(e) = reaction_add(&ctx, &add_reaction) {
            let author = {
                match add_reaction.user(&ctx) {
                    Ok(user) => user.name,
                    Err(_) => ":User Not Found:".to_owned(),
                }
            };
            match e {
                ReactionError::NoRows => return,
                ReactionError::ErrorFindingGuildMember(e) => {
                    warn!("Error finding {} in guild: {}", author, e);
                }
                ReactionError::ErrorAddingRole => {
                    warn!("Error adding role to {}", author);
                }
                err => error!("{}", err),
            }
        }
    }
    fn reaction_remove(&self, ctx: Context, removed_reaction: Reaction) {
        if let Err(e) = reaction_remove(&ctx, &removed_reaction) {
            let author = {
                match removed_reaction.user(&ctx) {
                    Ok(user) => user.name,
                    Err(_) => ":User Not Found:".to_owned(),
                }
            };
            match e {
                ReactionError::NoRows => return,
                ReactionError::ErrorFindingGuildMember(e) => {
                    warn!("Error finding {} in guild: {}", author, e);
                }
                ReactionError::ErrorAddingRole => {
                    warn!("Error removing role from {}", author);
                }
                err => error!("{}", err),
            }
        }
    }
}

#[group]
#[commands(ping, math, precision_math)]
/// A general grouping of commands
struct General;

#[group]
#[commands(about, user, avatar, server)]
/// Information commands, they give you information about things
struct Info;

#[group]
#[owners_only]
#[commands(quit, say, update)]
/// Commands that can only be ran by the owner of the bot
struct Owners;

#[group]
// This allows the bot owner to override certain permission checks
// Intended to be a temp option, while the bot is in pre 1.0 development
#[owner_privilege]
#[commands(prefix, reaction_add)]
/// Commands to assist with adminstrating a server
struct Admin;

#[help]
#[lacking_ownership = "hide"]
#[lacking_role = "hide"]
#[lacking_permissions = "strike"]
#[wrong_channel = "Strike"]
fn my_help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::plain(context, msg, args, &help_options, groups, owners)
}

fn dynamic_prefix(ctx: &mut Context, msg: &Message) -> Option<String> {
    // Make sure we can actually get the guild_id, if not there's
    // no point to trying to find the prefix. Also means we can use
    // unwrap for this later on, since we Guard check it's Some() here
    msg.guild_id?;

    // Try to keep the time we have the data lock as short as possible, since we intend to get the dynamic prefix quickly,
    // most of our time will be spent aquiring the lock, so we just go ahead and grab out everything we might need at once.
    let (hashmap_lock, fancy_db, runtime_lock) = {
        let data = ctx.data.try_read()?;
        let hashmap_lock = Arc::clone(data.get::<PrefixHashMapContainer>()?);
        let fancy_db = Arc::clone(data.get::<PoolContainer>()?);
        let runtime_lock = Arc::clone(data.get::<TokioContainer>()?);
        (hashmap_lock, fancy_db, runtime_lock)
    };

    // We hold the lock for the rest of the function, as if the guild's prefix is not
    // in the hashmap, after asking the db for the prefix, we'll want to insert that Into
    // the hashmap regardless, so it makes little sense to hold the lock for a shorter period
    // and aquire it again after the db responds.
    let mut hashmap = hashmap_lock.lock().ok()?;

    if let Some(prefix) = hashmap.get(msg.guild_id?.as_u64()) {
        return Some(prefix.clone());
    }

    let database_prefix = {
        let data = {
            let mut runtime = runtime_lock.try_lock().ok()?;
            runtime
                .block_on(
                    sqlx::query!(
                        "SELECT prefix FROM guild WHERE id = $1",
                        *msg.guild_id.unwrap().as_u64() as i64
                    )
                    .fetch_optional(fancy_db.pool()),
                )
                .ok()?
        };
        data?.prefix
    };

    hashmap.insert(*msg.guild_id?.as_u64(), database_prefix.clone());
    Some(database_prefix)
}

fn main() {
    dotenv::dotenv().ok();
    sentry::integrations::env_logger::init(None, Default::default());

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
            match std::env::var("DISCORD_TOKEN") {
                Ok(token) => token,
                Err(_) => settings
                    .get_str("token")
                    .expect("No token/token value set in Settings file"),
            }
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

    let tokio_runtime = Arc::new(Mutex::new(
        tokio::runtime::Runtime::new().expect("Couldn't start tokio runtime"),
    ));

    let pool = Arc::new(FancyPool::new(Arc::clone(&tokio_runtime)));

    let _guard = sentry::init((
        "https://c667c4bf6a704b0f802fa075c98f8c03@sentry.io/1340627",
        sentry::ClientOptions {
            max_breadcrumbs: 50,
            environment: Some(enviroment.into()),
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));

    sentry::integrations::panic::register_panic_handler();

    let mut client = Client::new(&token, Handler).expect("Err creating client");

    let prefix_hash_arc: Arc<Mutex<HashMap<u64, String>>> = Arc::new(Mutex::new(HashMap::new()));

    {
        let mut data = client.data.write();
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<SettingsContainer>(Arc::clone(&config));
        data.insert::<TokioContainer>(Arc::clone(&tokio_runtime));
        data.insert::<PoolContainer>(Arc::clone(&pool));
        data.insert::<PrefixHashMapContainer>(Arc::clone(&prefix_hash_arc));
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
                    .dynamic_prefix(|ctx: &mut Context, msg: &Message| {
                        // Seperate function so dynamic prefix can look cleaner
                        // (this allows for us to use return None, when dynamic_prefix
                        // has no results, Allowing us here, to use a "default" prefix
                        // in the case that it is None for any reason)
                        if let Some(prefix) = dynamic_prefix(ctx, msg) {
                            return Some(prefix);
                        }
                        Some("a.".to_string())
                    })
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
                DispatchError::IgnoredBot => {},
                DispatchError::NotEnoughArguments{ min, given} => {
                    let _ = message.channel_id.say(&context.http, format!("You did not provide enough arguments for this command, Minimum arguments are {}, you provided {}.", min, given));
                }
                _ => warn!("Dispatch Error: {} failed: {:?}", message.content, error),
            })
            // TODO: Better error handling
            .after(|context, message, command_name, error| if let Err(why) = error {
                    let _ = message.channel_id.say(&context.http, format!("The command {} has errored: ``{}``\nPlease try again later", command_name, why.0));
                    warn!("Command `{}` triggered by `{}` has errored: \n{}", command_name, message.author.tag(), why.0);
            })
            .help(&MY_HELP)
            .group(&GENERAL_GROUP)
            .group(&OWNERS_GROUP)
            .group(&INFO_GROUP)
            .group(&ADMIN_GROUP)
    );

    if let Err(why) = client.start_autosharded() {
        error!("Client error: {:?}", why);
    }
}
