CREATE TABLE custom_emojis (
    id UUID PRIMARY KEY,
    shortcode TEXT NOT NULL,
    domain TEXT,
    remote_id TEXT UNIQUE NULLS NOT DISTINCT,
    media_attachment_id UUID NOT NULL,
    endorsed BOOLEAN NOT NULL DEFAULT FALSE,

    -- UNIQUE constraints
    UNIQUE (shortcode, domain),
    
    -- Foreign key constraints
    FOREIGN KEY (media_attachment_id) REFERENCES media_attachments(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE posts_custom_emojis (
    post_id UUID NOT NULL,
    custom_emoji_id UUID NOT NULL,
    PRIMARY KEY (post_id, custom_emoji_id),

    -- Foreign key constraints
    FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (custom_emoji_id) REFERENCES custom_emojis(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE INDEX "idx-custom_emojis-remote_id" ON custom_emojis (remote_id);
CREATE INDEX "idx-custom_emojis-shortcode" ON custom_emojis (shortcode);
CREATE INDEX "idx-custom_emojis-domain" ON custom_emojis (domain);
