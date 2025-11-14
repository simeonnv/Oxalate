-- Add migration script here
CREATE TABLE IF NOT EXISTS Logs (
    log_id UUID PRIMARY KEY,
    log_level TEXT NOT NULL,
    body TEXT NOT NULL,
    device_id TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP  
);

CREATE INDEX IF NOT EXISTS idx_logs_body ON Logs (body);
CREATE INDEX IF NOT EXISTS idx_logs_device_id ON Logs (device_id);
CREATE INDEX IF NOT EXISTS idx_logs_log_level ON Logs (log_level);
CREATE INDEX IF NOT EXISTS idx_logs_created_at ON Logs (created_at);
