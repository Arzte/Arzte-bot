use core::built_info;

command!(about(_ctx, msg, _args) {
    msg.channel_id.say(format!("{} (v ``{}``) is a small utility bot, developed by <@77812253511913472>, with help from serenity and it's resources.\n\n\
    There are currently no set plans for this bot", built_info::PKG_NAME, built_info::PKG_VERSION))?;
});

use serenity::model::id::GuildId;
use serenity::CACHE;

command!(guild(_ctx, msg, args) {
    let g = match GuildId(args.single::<u64>()?).to_partial_guild() {
        Ok(partial_guild) => partial_guild,
        Err(_arg_error) => {
            msg.channel_id.say(":no_entry_sign: Invalid server/Not Available")?;
            return Ok(())
        },
    };
    let guild_lock = CACHE.read().guild(&g.id);
    if let Some(guild_lock) = guild_lock {
        let guildd = guild_lock.read();
        msg.channel_id.send_message(|m| {
            m.embed(|e| {
                let mut e = e
                        .title(&g.name)
                        .field("ID", &g.id, true)
                        .field("Name", &g.name, true)
                        .field("Owner", format!("<@{}>", &g.owner_id.as_u64()), true)
                        .field("Region", &g.region, true)
                        .field("Members", guildd.members.len(), true)
                        .field("Created on", &g.id.created_at().format("%a, %d %h %Y @ %H:%M:%S").to_string(), true)
                        .field("Roles", &g.roles.len(), true)
                        .field("Emojis", &g.emojis.len(), true);
                if let Some(icon_url) = &g.icon_url() {
                    e = e.thumbnail(&icon_url).author(|a| a.name(&g.name).icon_url(&icon_url));
                }

                e
            })
        }).unwrap();
    };
});
