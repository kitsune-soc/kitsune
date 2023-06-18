CREATE TABLE link_previews (
    url TEXT PRIMARY KEY,
    embed_data JSONB NOT NULL
);

ALTER TABLE posts
    ADD COLUMN link_preview_url TEXT REFERENCES link_previews(url)
        ON DELETE SET NULL ON UPDATE CASCADE;
