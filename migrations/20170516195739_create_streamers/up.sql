CREATE TABLE streamers (
    id INTEGER PRIMARY KEY NOT NULL,
    server_id VARCHAR(255) NOT NULL,
    channel_id VARCHAR(255) NOT NULL,
    twitch_nick VARCHAR(255) NOT NULL,
    is_online BOOLEAN NOT NULL DEFAULT false
);
