-- Add migration script here
CREATE TABLE IF NOT EXISTS Keylog (
    keylog_id UUID PRIMARY KEY,
    device_id TEXT NOT NULL,
    key TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_keylog_created_at ON Keylog (created_at DESC);

CREATE INDEX IF NOT EXISTS idx_keylog_key ON Keylog (key);