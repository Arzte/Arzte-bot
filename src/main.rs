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
    core::structs::{
        PoolContainer,
        PrefixHashMapContainer,
        SettingsContainer,
        ShardManagerContainer,
        TokioContainer,
    },
};

use sqlx::PgPool;

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }

    fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        // Temporarly limit to one server
        // TODO: Generize this so it can work in other servers
        if add_reaction.channel_id.as_u64() != &675_885_303_525_015_582 {
            return;
        }
        let reaction = add_reaction.clone();
        let guild_lock = {
            match reaction.channel(&ctx).unwrap().guild() {
                Some(guild_channel) => match guild_channel.read().guild(&ctx) {
                    Some(v) => v,
                    None => return,
                },
                None => return,
            }
        };
        let guild = guild_lock.read();

        let mut guild_member = guild.member(&ctx, reaction.user_id).unwrap();

        let emoji_name = {
            match add_reaction.emoji {
                ReactionType::Custom { name, .. } => name.unwrap(),
                ReactionType::Unicode(name) => name,
                _ => "".to_owned(),
            }
        };

        match emoji_name.as_ref() {
            "⛏\u{fe0f}" => {
                if let Err(error) = guild_member.add_role(&ctx, 675_944_554_989_486_105) {
                    warn!("Unable to give role: {:?}", error);
                }
            }
            "🎞\u{fe0f}" => {
                if let Err(error) = guild_member.add_role(&ctx, 675_944_444_868_034_613) {
                    warn!("Unable to give role: {:?}", error);
                }
            }
            "🔔" => {
                if let Err(error) = guild_member.add_role(&ctx, 677_524_235_463_426_051) {
                    warn!("Unable to give role: {:?}", error);
                }
            }
            v => {
                log::debug!("{:?}", v);
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
#[commands(quit, update, prefix)]
/// Commands that can only be ran by the owner of the bot
struct Owners;

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

    let data = match ctx.data.try_read() {
        Some(v) => v,
        None => return None,
    };

    let database_prefix = || {
        let mut db = match data.get::<PoolContainer>() {
            Some(pg_connection) => pg_connection,
            None => return None,
        };
        if let Some(runtime_lock) = data.get::<TokioContainer>() {
            if let Ok(mut runtime) = Arc::clone(runtime_lock).try_lock() {
                if let Ok(prefix) = runtime.block_on(
                    sqlx::query!(
                        "SELECT prefix FROM guild WHERE id = $1",
                        *msg.guild_id.unwrap().as_u64() as i64
                    )
                    .fetch_optional(&mut db),
                ) {
                    if let Some(data) = prefix {
                        return Some(data.prefix);
                    }
                }
            }
        }
        None
    };

    if let Some(hashmap_lock) = data.get::<PrefixHashMapContainer>() {
        if let Ok(mut hashmap) = Arc::clone(hashmap_lock).try_lock() {
            if let Some((_, prefix)) = hashmap.get_key_value(msg.guild_id.unwrap().as_u64()) {
                return Some(prefix.clone());
            }
            let prefix = database_prefix();
            if let Some(value) = prefix.clone() {
                hashmap.insert(*msg.guild_id.unwrap().as_u64(), value);
            }
            return prefix;
        }
    }

    database_prefix()
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

    let tokio_runtime = Arc::new(Mutex::new(
        tokio::runtime::Runtime::new().expect("Couldn't start tokio runtime"),
    ));

    let pool = tokio_runtime
        .try_lock()
        .expect("Unable to get runtime lock to start database pool")
        .block_on(async {
            PgPool::new(
                &std::env::var("DATABASE_URL").expect("DATABASE_URL enviroment variable not set"),
            )
            .await
            .expect("unable to connect to db")
        });

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
        data.insert::<PoolContainer>(pool);
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
                        Some("!".to_string())
                        
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
    );

    if let Err(why) = client.start_autosharded() {
        error!("Client error: {:?}", why);
    }
}
