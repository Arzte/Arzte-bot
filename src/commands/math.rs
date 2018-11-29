command!(math(_ctx, msg, args) {
    let value = meval::eval_str(&args.full());

    if let Err(rr) = value {
        let _ = msg.channel_id.say(format!("```{:?}```", rr));
        return Ok(())
    };
    if let Ok(orr) = value {
        let _ = msg.channel_id.say(orr);
        return Ok(())
    };
});
