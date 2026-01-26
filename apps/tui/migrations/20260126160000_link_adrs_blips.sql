-- Add up migration script here
ALTER TABLE adr_log ADD COLUMN blip_name TEXT NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS adr_log_title_timestamp_unique
    ON adr_log(title, timestamp);

ALTER TABLE blip ADD COLUMN adr_id INTEGER;

-- Add down migration script here
-- SQLite does not support DROP COLUMN without table rebuild.
