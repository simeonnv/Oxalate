CREATE TABLE IF NOT EXISTS ProxyResourseUsage(
    proxy_resourse_usage_id UUID PRIMARY KEY,
    device_id TEXT NOT NULL,
    ram_usage REAL NOT NULL,
    cpu_usage REAL NOT NULL,
    net_usage_bytes BIGINT NOT NULL CHECK (net_usage_bytes > 0),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP  
);

CREATE INDEX IF NOT EXISTS idx_proxy_resourse_usage_created_at ON ProxyResourseUsage (created_at);
CREATE INDEX IF NOT EXISTS idx_proxy_resourse_usage_net_usage_bytes ON ProxyResourseUsage (net_usage_bytes);
