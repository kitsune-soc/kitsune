CREATE TABLE link_previews (
    url TEXT PRIMARY KEY,
    embed_data JSONB NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE posts
    ADD COLUMN link_preview_url TEXT REFERENCES link_previews(url)
        ON DELETE SET NULL ON UPDATE CASCADE;

SELECT diesel_manage_updated_at('link_previews');
