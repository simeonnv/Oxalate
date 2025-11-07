CREATE TABLE IF NOT EXISTS Uptime (
    uptime_id UUID PRIMARY KEY,
    device_id TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_uptime_uptime_id ON Uptime (uptime_id);

CREATE INDEX IF NOT EXISTS idx_uptime_device_id ON Uptime (device_id);