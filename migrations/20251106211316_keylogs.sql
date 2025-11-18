CREATE TABLE IF NOT EXISTS Keylogs (
    id UUID PRIMARY KEY,
    device_machine_id TEXT NOT NULL,
    key TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,

    CONSTRAINT fk_device FOREIGN KEY(device_machine_id) REFERENCES Devices(machine_id)
);

CREATE INDEX IF NOT EXISTS idx_keylogs_created_at ON Keylogs (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_keylogs_device_machine_id ON Keylogs (device_machine_id);
