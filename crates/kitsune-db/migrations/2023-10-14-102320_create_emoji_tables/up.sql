CREATE TABLE custom_emojis (
    id UUID PRIMARY KEY,
    shortcode TEXT NOT NULL,
    remote_id TEXT UNIQUE,
    media_attachments_id UUID NOT NULL,
    custom_emoji_packs_id UUID NOT NULL,

    -- UNIQUE constraints
    UNIQUE (shortcode, custom_emoji_packs_id),
    
    -- Foreign key constraints
    FOREIGN KEY (media_attachment_id) REFERENCES media_attachments(id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (custom_emoji_packs_id) REFERENCES custom_emoji_packs(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE custom_emoji_packs (
    id UUID PRIMARY KEY,
    domain TEXT UNIQUE,
    code TEXT UNIQUE,
    globally_enabled BOOLEAN DEFAULT FALSE,

    -- UNIQUE constraints
    UNIQUE (domain, code)
);

CREATE TABLE accounts_preferences_emoji_packs (
    accounts_preferences_id UUID NOT NULL,
    custom_emoji_packs_id UUID NOT NULL,
    PRIMARY KEY (accounts_preferences_id, custom_emoji_packs_id),

    -- Foreign key constraints
    FOREIGN KEY (accounts_preferences_id) REFERENCES accounts_preferencecs(id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (custom_emoji_packs_id) REFERENCES custom_emoji_packs(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE INDEX "idx-custom_emojis-remote_id" ON custom_emojis (remote_id);
CREATE INDEX "idx-custom_emojis-shortcode" ON custom_emojis (shortcode);
