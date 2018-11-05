command!(multiply(_ctx, msg, args) {
    let one = args.single::<f64>()?;
    let two = args.single::<f64>()?;

    let product = one * two;

    let _ = msg.channel_id.say(product);
});
