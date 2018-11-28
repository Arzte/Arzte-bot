use calc::eval;

command!(math(_ctx, msg, args) {
    let eval = eval(&args.full())?;

    let _ = msg.channel_id.say(eval);
});
