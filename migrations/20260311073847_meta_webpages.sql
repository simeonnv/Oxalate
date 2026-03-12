CREATE TABLE IF NOT EXISTS MetaWebpages (
    -- id UUID PRIMARY KEY,
    url TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    keywords TEXT NOT NULL,
    search_engine TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP  
);

CREATE INDEX IF NOT EXISTS idx_meta_webpages_url ON MetaWebpages (url);  
CREATE INDEX ON MetaWebpages USING bm25 (url, keywords) WITH (key_field='url');
