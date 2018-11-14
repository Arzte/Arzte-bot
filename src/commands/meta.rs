extern crate chrono;

use self::chrono::Duration;
use core::structs::ShardManagerContainer;
use serenity::client::bridge::gateway::ShardId;

command!(ping(ctx, msg) {
    // This is done to find the time difference between when sending the the 
    // message and when discord states the message was sent
    let start = msg.timestamp.timestamp_millis();
    let mut message = msg.channel_id.say("Pong!")?;
    let timestamp = message.timestamp.timestamp_millis() - start;

    // The shard manager is an interface for mutating, stopping, restarting, and
    // retrieving information about shards.
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

    // Shards are backed by a "shard runner" responsible for processing events
    // over the shard, so we'll get the information about the shard runner for
    // the shard this command was sent over.
    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            let _ = msg.reply("No shard found");

            return Ok(());
        },
    };
    // The latency function for a shard returns a option, so handle that
    let latency = match runner.latency {
        Some(latency) => {
            // The from_std() conversion function from crono can return an error
            // if the duration is out of bounds for u64. While this is unlikely 
            // for Discord shard latency to be that high, we're going to handle
            // the result given like it might anyways.
            match Duration::from_std(latency) {
                Ok(milli) => format!("{}ms", milli.num_milliseconds()),
                Err(_error) => "result is to high to calculate.".to_string()
            }
        },
        // Sometimes there's no latency reported on the shard (yeah idk how that 
        // works tbh) but in that case, we want to report that there is none, so,
        // that's what this is for. 
        None => "0ms".to_string()
    };

    let string = format!("Pong! \n**```prolog\n   Message Latency: {}ms, \n     Shard Latency: {}\n```**", timestamp, latency);
    message.edit(|m| m.content(string))?;
});
