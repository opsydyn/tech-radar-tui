-- Add up migration script here
CREATE TABLE blip (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    ring TEXT,
    quadrant TEXT,
    tag TEXT,
    description TEXT,
    created TEXT NOT NULL,
    hasAdr BOOLEAN DEFAULT FALSE
);

-- Add down migration script here
DROP TABLE IF EXISTS blip;
