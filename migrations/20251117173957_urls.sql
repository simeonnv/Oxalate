CREATE TABLE IF NOT EXISTS Urls (
    url TEXT PRIMARY KEY,
    last_scanned TIMESTAMP,
    device_machine_id TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,  

    CONSTRAINT fk_device FOREIGN KEY(device_machine_id) REFERENCES Devices(machine_id)
);

CREATE INDEX IF NOT EXISTS idx_urls_url ON Urls (url);
CREATE INDEX IF NOT EXISTS idx_urls_device_machine_id ON Urls (device_machine_id);
