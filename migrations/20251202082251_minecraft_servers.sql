CREATE TABLE IF NOT EXISTS MinecraftServers (
    url TEXT PRIMARY KEY,
    last_scanned TIMESTAMP,
    device_machine_id TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    online_when_scraped BOOLEAN NOT NULL,
    online_players_count INT NOT NULL,
    max_online_players INT NOT NULL,
    players TEXT[],
    server_version TEXT NOT NULL,
    mods TEXT[],


    CONSTRAINT fk_device FOREIGN KEY (device_machine_id) REFERENCES Devices (machine_id)
);
