CREATE TABLE IF NOT EXISTS Webpages (
    webpage_id UUID PRIMARY KEY,
    body TEXT NOT NULL,
    headers  jsonb NOT NULL DEFAULT '{}',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP  
);

CREATE INDEX idx_webpages_headers_gin ON Webpages USING GIN (headers jsonb_path_ops);
CREATE INDEX idx_webpages_body ON Webpages (body);
