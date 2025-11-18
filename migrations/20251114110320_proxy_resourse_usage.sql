CREATE TABLE IF NOT EXISTS ProxyResourseUsage(
    id UUID PRIMARY KEY,
    device_machine_id TEXT NOT NULL,
    ram_usage REAL NOT NULL,
    cpu_usage REAL NOT NULL,
    net_usage_bytes BIGINT NOT NULL CHECK (net_usage_bytes > 0),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,  

    CONSTRAINT fk_device FOREIGN KEY(device_machine_id) REFERENCES Devices(machine_id)
);

CREATE INDEX IF NOT EXISTS idx_proxy_resourse_usage_created_at ON ProxyResourseUsage (created_at);
CREATE INDEX IF NOT EXISTS idx_proxy_resourse_usage_device_machine_id ON ProxyResourseUsage (device_machine_id);
CREATE INDEX IF NOT EXISTS idx_proxy_resourse_usage_net_usage_bytes ON ProxyResourseUsage (net_usage_bytes);
