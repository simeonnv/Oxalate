CREATE TABLE IF NOT EXISTS Uptime (
    id UUID PRIMARY KEY,
    device_machine_id TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT fk_device FOREIGN KEY(device_machine_id) REFERENCES Devices(machine_id)
);

CREATE INDEX IF NOT EXISTS idx_uptime_uptime_id ON Uptime (id);
CREATE INDEX IF NOT EXISTS idx_uptime_device_machine_id ON Uptime (device_machine_id);
