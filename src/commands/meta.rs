extern crate chrono;

use self::chrono::Duration;
use core::structs::ShardManagerContainer;
use serenity::client::bridge::gateway::ShardId;

command!(ping(ctx, msg) {
    let start = msg.timestamp.timestamp_millis();
    let mut message = msg.channel_id.say("Pong!")?;
    let timestamp = message.timestamp.timestamp_millis() - start;

    let data = ctx.data.lock();

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            let _ = msg.reply("There was a problem getting the shard manager");

            return Ok(());
        },
    };

    let manager = shard_manager.lock();
    let runners = manager.runners.lock();

    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            let _ = msg.reply("No shard found");

            return Ok(());
        },
    };
    let latency = match runner.latency {
        Some(latency) => {
            match Duration::from_std(latency) {
                Ok(milli) => format!("{}ms", milli.num_milliseconds()),
                Err(_error) => "result is to high to calculate.".to_string()
            }
        },
        None => "0ms".to_string()
    };

    let string = format!("Pong! \n**```prolog\n   Message Latency: {}ms, \n     Shard Latency: {}\n```**", timestamp, latency);
    message.edit(|m| m.content(string))?;
});
