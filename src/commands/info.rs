use crate::core::built_info;
use serenity::{
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    model::{
        id::GuildId,
        prelude::Message,
    },
    prelude::Context,
};

#[command]
fn about(ctx: &mut Context, msg: &Message) -> CommandResult {
    // TODO: Implment a working way to detect and fill in the current application version, rather than updating it by hand.
    //       (GitHub would work, however if for some reason those differ, it could be problematic.)
    let _ = msg.channel_id.say(&ctx.http, format!("{} (v ``{}``) is a small utility bot, developed by <@77812253511913472>, with help from serenity and it's resources.\n\n\
    There are currently no set plans for this bot", built_info::PKG_NAME, built_info::PKG_VERSION));
    Ok(())
}

#[command]
fn guild(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = if args.is_empty() {
        if let Some(gid) = msg.guild_id {
            *gid.as_u64()
        } else {
            msg.channel_id.say(
                &ctx.http,
                "I was unable to get the current guild id, try again later.",
            )?;
            return Ok(());
        }
    } else {
        args.single::<u64>()?
    };
    let g = match GuildId(guild_id).to_partial_guild(&ctx.http) {
        Ok(partial_guild) => partial_guild,
        Err(_arg_error) => {
            msg.channel_id
                .say(&ctx.http, ":no_entry_sign: Invalid server/Not Available")?;
            return Ok(());
        }
    };
    let guild_lock = &ctx.cache.read().guild(&g.id);
    if let Some(guild_lock) = guild_lock {
        let guildd = guild_lock.read();
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    let mut e = e
                        .title(&g.name)
                        .field("ID", &g.id, false)
                        .field("Name", &g.name, true)
                        .field("Owner", format!("<@{}>", &g.owner_id.as_u64()), true)
                        .field("Region", &g.region, true)
                        .field("Members", guildd.members.len(), true)
                        .field(
                            "Created on",
                            &g.id
                                .created_at()
                                .format("%a, %d %h %Y @ %H:%M:%S")
                                .to_string(),
                            true,
                        )
                        .field("Roles", &g.roles.len(), true)
                        .field("Emojis", &g.emojis.len(), true);
                    if let Some(icon_url) = &g.icon_url() {
                        e = e
                            .thumbnail(&icon_url)
                            .author(|a| a.name(&g.name).icon_url(&icon_url));
                    }

                    e
                })
            })
            .unwrap();
    };
    Ok(())
}
