-- Add up migration script here
CREATE TABLE adr_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    timestamp TEXT NOT NULL
);

-- Add down migration script here
DROP TABLE IF EXISTS adr_log;
