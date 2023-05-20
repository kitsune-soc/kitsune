CREATE TABLE media_attachments (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL,
    content_type TEXT NOT NULL,
    description TEXT,
    blurhash TEXT,
    file_path TEXT,
    remote_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Foreign key constraints
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE posts_media_attachments (
    post_id UUID NOT NULL,
    media_attachment_id UUID NOT NULL,
    PRIMARY KEY (post_id, media_attachment_id),

    -- Foreign key constraints
    FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (media_attachment_id) REFERENCES media_attachments(id) ON DELETE CASCADE ON UPDATE CASCADE
);

ALTER TABLE accounts
    ADD avatar_id UUID,
    ADD FOREIGN KEY (avatar_id) REFERENCES media_attachments(id)
    ON DELETE CASCADE
    ON UPDATE CASCADE;

ALTER TABLE accounts
    ADD header_id UUID,
    ADD FOREIGN KEY (header_id) REFERENCES media_attachments(id)
    ON DELETE CASCADE
    ON UPDATE CASCADE;

SELECT diesel_manage_updated_at('media_attachments');
