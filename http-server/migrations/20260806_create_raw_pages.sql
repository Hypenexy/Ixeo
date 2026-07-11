-- migrations/20260806create_raw_pages.sql
CREATE TABLE IF NOT EXISTS raw_pages (
    id SERIAL PRIMARY KEY,
    url TEXT UNIQUE NOT NULL,
    title TEXT,
    html_content TEXT,
    crawled_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
