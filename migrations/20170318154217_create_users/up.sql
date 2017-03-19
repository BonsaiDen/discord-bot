CREATE TABLE users (
    id INTEGER PRIMARY KEY NOT NULL,
    server_id VARCHAR(255) NOT NULL,
    nickname VARCHAR(255) NOT NULL,
    is_admin BOOLEAN NOT NULL DEFAULT false,
    is_uploader BOOLEAN NOT NULL DEFAULT false,
    is_banned BOOLEAN NOT NULL DEFAULT false
);
