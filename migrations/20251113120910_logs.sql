-- Add migration script here
CREATE TABLE IF NOT EXISTS Logs (
    id UUID PRIMARY KEY,
    log JSONB NOT NULL,
    device_machine_id TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT fk_device FOREIGN KEY(device_machine_id) REFERENCES Devices(machine_id)
);

CREATE INDEX IF NOT EXISTS idx_logs_device_machine_id ON Logs (device_machine_id);
CREATE INDEX IF NOT EXISTS idx_logs_created_at ON Logs (created_at);
