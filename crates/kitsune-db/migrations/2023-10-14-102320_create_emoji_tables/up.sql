CREATE TABLE custom_emojis (
    id UUID PRIMARY KEY,
    shortcode TEXT NOT NULL,
    domain TEXT,
    remote_id TEXT UNIQUE NULLS NOT DISTINCT,
    media_attachment_id UUID NOT NULL,
    global BOOLEAN NOT NULL DEFAULT FALSE,

    -- UNIQUE constraints
    UNIQUE (shortcode, domain),
    
    -- Foreign key constraints
    FOREIGN KEY (media_attachment_id) REFERENCES media_attachments(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE INDEX "idx-custom_emojis-remote_id" ON custom_emojis (remote_id);
CREATE INDEX "idx-custom_emojis-shortcode" ON custom_emojis (shortcode);
CREATE INDEX "idx-custom_emojis-domain" ON custom_emojis (domain);
