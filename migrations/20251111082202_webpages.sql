CREATE TABLE IF NOT EXISTS Webpages (
    id UUID PRIMARY KEY,
    url TEXT NOT NULL,
    compressed_body BYTEA NOT NULL,
    keywords TEXT NOT NULL,
    headers  jsonb NOT NULL DEFAULT '{}',
    device_machine_id TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,  

    CONSTRAINT fk_device FOREIGN KEY(device_machine_id) REFERENCES Devices(machine_id)
);

CREATE INDEX IF NOT EXISTS idx_webpages_id ON Webpages (id);  
CREATE INDEX IF NOT EXISTS idx_webpages_url ON Webpages (url);  
CREATE INDEX IF NOT EXISTS idx_webpages_device_machine_id ON Webpages (device_machine_id);  
CREATE INDEX ON Webpages USING GIN (headers jsonb_path_ops);
CREATE INDEX ON Webpages USING bm25 (id, keywords) WITH (key_field='id');
