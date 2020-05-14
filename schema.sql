CREATE TABLE IF NOT EXISTS guild (
    id bigint PRIMARY KEY NOT NULL UNIQUE,
    /* TODO make varchar(100) instead of text */
    name text NOT NULL,
    updated_at timestamptz,
    prefix text NOT NULL DEFAULT 'a.'
);

CREATE TABLE IF NOT EXISTS reaction_roles (
    guild_id bigint NOT NULL,
    role_id bigint UNIQUE PRIMARY KEY NOT NULL,
    message_id bigint NOT NULL,
    name text NOT NULL,
    emoji_id bigint
);
