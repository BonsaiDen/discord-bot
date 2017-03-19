CREATE TABLE effects (
    id INTEGER PRIMARY KEY NOT NULL,
    server_id VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    uploader VARCHAR(255) NOT NULL,
    peak_db FLOAT NOT NULL DEFAULT 0,
    duration_ms INTEGER NOT NULL DEFAULT 0,
    silent_start_samples INTEGER NOT NULL DEFAULT 0,
    silent_end_samples INTEGER NOT NULL DEFAULT 0,
    transcript TEXT NOT NULL
);
