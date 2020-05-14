CREATE TABLE IF NOT EXISTS guild (
    id bigint PRIMARY KEY NOT NULL UNIQUE,
    /* TODO make varchar(100) instead of text */
    name text NOT NULL,
    updated_at timestamptz,
    prefix text NOT NULL DEFAULT 'a.'
);

