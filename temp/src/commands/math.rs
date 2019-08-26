use asciimath::eval;

command!(math(_ctx, msg, args) {
    let scope_empty = scope!{};
    let value = eval(&args.full(), &scope_empty);

    if let Err(rr) = value {
        let _ = msg.channel_id.say(format!("```{:?}```", rr));
        return Ok(())
    };
    if let Ok(orr) = value {
        let _ = msg.channel_id.say(orr);
        return Ok(())
    };
});
