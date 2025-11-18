CREATE TABLE IF NOT EXISTS Devices (
    machine_id TEXT PRIMARY KEY,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP  
);

CREATE INDEX IF NOT EXISTS idx_devices_machine_id ON Devices (machine_id);

