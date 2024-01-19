-- Schema for utility functions and collations, to keep the "public" schema clean
CREATE SCHEMA kitsune;

-- Unicode collation that effectively ignores all accent and case differences
-- We use this on our username columns to achieve case insensitivity and prevent impersonation through accent characters
CREATE COLLATION kitsune.ignore_accent_case (
    provider = icu,
    deterministic = false,
    locale = 'und-u-ks-level1'
    );

-- This enum is automatically updated when starting Kitsune
-- It gets all the supported ISO-639-3 codes pushed into it
--
-- Supported languages are all languages with an assigned ISO-639-1 code + whatever is supported by our language detection backends
-- Note: Values are *never* deleted from this enum. This enum is purely append-only.
CREATE TYPE kitsune.language_iso_code AS ENUM();

-- This function is responsible for mapping the ISO-639-3 code to the associated "regconfig"
-- It is used inside the stored "tsvector" columns to automatically provide language aware tokenization for the full-text search
--
-- Purely a temporary function. This function is overwritten on each start-up of Kitsune using freshly read metadata
-- We need this for the migrations to succeed
CREATE FUNCTION kitsune.iso_code_to_language(kitsune.language_iso_code)
    RETURNS regconfig
AS
$$
SELECT 'english'::regconfig
$$
    LANGUAGE SQL IMMUTABLE;

CREATE TABLE accounts
(
    id                      UUID PRIMARY KEY,
    display_name            TEXT,
    note                    TEXT,
    -- Use special collation to ignore case and accent differences
    username                TEXT                                                                  NOT NULL COLLATE kitsune.ignore_accent_case,
    locked                  BOOLEAN                                                               NOT NULL,
    local                   BOOLEAN                                                               NOT NULL,
    domain                  TEXT                                                                  NOT NULL,
    actor_type              INTEGER                                                               NOT NULL,
    url                     TEXT UNIQUE                                                           NOT NULL,

    -- ActivityPub-specific data
    featured_collection_url TEXT,
    followers_url           TEXT,
    following_url           TEXT,
    inbox_url               TEXT,
    outbox_url              TEXT,
    shared_inbox_url        TEXT,
    public_key_id           TEXT                                                                  NOT NULL UNIQUE,
    public_key              TEXT                                                                  NOT NULL,

    created_at              TIMESTAMPTZ                                                           NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ                                                           NOT NULL DEFAULT NOW(),

    -- Generated full-text search column
    account_ts              TSVECTOR GENERATED ALWAYS AS (
                                setweight(to_tsvector('simple', COALESCE(display_name, '')) ||
                                          to_tsvector('simple', username), 'A') ||
                                setweight(to_tsvector('simple', COALESCE(note, '')), 'B')) STORED NOT NULL
);

ALTER TABLE accounts
    ADD CONSTRAINT "constr-unique-username-domain"
        UNIQUE (username, domain);

CREATE INDEX "idx-accounts-account_ts" ON accounts USING GIN (account_ts);

CREATE TABLE accounts_follows
(
    id          UUID PRIMARY KEY,
    account_id  UUID        NOT NULL,
    follower_id UUID        NOT NULL,
    approved_at TIMESTAMPTZ,
    url         TEXT        NOT NULL UNIQUE,
    notify      BOOLEAN     NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- UNIQUE constraints
ALTER TABLE accounts_follows
    ADD CONSTRAINT "constr-unique-account_id-follower_id"
        UNIQUE (account_id, follower_id);

-- Foreign key constraints
ALTER TABLE accounts_follows
    ADD CONSTRAINT "const-foreign-account_id"
        FOREIGN KEY (account_id) REFERENCES accounts (id) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE accounts_follows
    ADD CONSTRAINT "constr-foreign-follower_id"
        FOREIGN KEY (follower_id) REFERENCES accounts (id) ON DELETE CASCADE ON UPDATE CASCADE;

CREATE INDEX "idx-accounts_follows-account_id" ON accounts_follows (account_id);
CREATE INDEX "idx-accounts_follows-follower_id" ON accounts_follows (follower_id);

CREATE TABLE accounts_preferences
(
    account_id               UUID PRIMARY KEY,
    notify_on_follow         BOOLEAN NOT NULL,
    notify_on_follow_request BOOLEAN NOT NULL,
    notify_on_repost         BOOLEAN NOT NULL,
    notify_on_post_update    BOOLEAN NOT NULL,
    notify_on_favourite      BOOLEAN NOT NULL,
    notify_on_mention        BOOLEAN NOT NULL
);

-- Foreign key constraints
ALTER TABLE accounts_preferences
    ADD CONSTRAINT "constr-foreign-account_id"
        FOREIGN KEY (account_id)
            REFERENCES accounts (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE;

CREATE TABLE users
(
    id                 UUID PRIMARY KEY,
    account_id         UUID        NOT NULL UNIQUE,
    oidc_id            TEXT UNIQUE,
    -- Use special collation to ignore case and accent differences
    username           TEXT        NOT NULL COLLATE kitsune.ignore_accent_case,
    email              TEXT        NOT NULL UNIQUE,
    password           TEXT UNIQUE,
    domain             TEXT        NOT NULL,
    private_key        TEXT        NOT NULL,

    -- Email confirmation
    confirmed_at       TIMESTAMPTZ,
    confirmation_token TEXT        NOT NULL,

    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- UNIQUE constraints
ALTER TABLE users
    ADD CONSTRAINT "constr-unique-username-domain"
        UNIQUE (username, domain);

ALTER TABLE users
    ADD CONSTRAINT "constr-unique-confirmation_token"
        UNIQUE (confirmation_token);

-- Foreign key constraints
ALTER TABLE users
    ADD CONSTRAINT "constr-foreign-account_id"
        FOREIGN KEY (account_id) REFERENCES accounts (id);

CREATE TABLE posts
(
    id               UUID PRIMARY KEY,
    account_id       UUID                                                                                                   NOT NULL,

    in_reply_to_id   UUID,
    reposted_post_id UUID,

    is_sensitive     BOOLEAN                                                                                                NOT NULL,
    subject          TEXT,

    content          TEXT                                                                                                   NOT NULL,
    content_source   TEXT                                                                                                   NOT NULL,
    content_lang     kitsune.language_iso_code                                                                              NOT NULL,

    visibility       INTEGER                                                                                                NOT NULL,
    is_local         BOOLEAN                                                                                                NOT NULL,
    url              TEXT                                                                                                   NOT NULL UNIQUE,

    created_at       TIMESTAMPTZ                                                                                            NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ                                                                                            NOT NULL DEFAULT NOW(),

    -- Generated full-text search column
    post_ts          TSVECTOR GENERATED ALWAYS AS (to_tsvector(kitsune.iso_code_to_language(content_lang),
                                                               COALESCE(subject, '')) ||
                                                   to_tsvector(kitsune.iso_code_to_language(content_lang), content)) STORED NOT NULL
);

-- Foreign key constraints
ALTER TABLE posts
    ADD CONSTRAINT "constr-foreign-account_id"
        FOREIGN KEY (account_id) REFERENCES accounts (id) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE posts
    ADD CONSTRAINT "constr-foreign-in_reply_to_id"
        FOREIGN KEY (in_reply_to_id) REFERENCES posts (id) ON DELETE SET NULL ON UPDATE CASCADE;

ALTER TABLE posts
    ADD CONSTRAINT "constr-foreign-reposted_post_id"
        FOREIGN KEY (reposted_post_id) REFERENCES posts (id) ON DELETE CASCADE ON UPDATE CASCADE;

CREATE INDEX "idx-posts-account_id" ON posts (account_id);
CREATE INDEX "idx-posts-in_reply_to_id" ON posts (in_reply_to_id);
CREATE INDEX "idx-posts-reposted_post_id" ON posts (reposted_post_id);
CREATE INDEX "idx-posts-visibility" ON posts (visibility);
CREATE INDEX "idx-posts-post_ts" ON posts USING GIN (post_ts);

CREATE TABLE posts_favourites
(
    id         UUID PRIMARY KEY,
    account_id UUID        NOT NULL,
    post_id    UUID        NOT NULL,
    url        TEXT        NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- UNIQUE constraints
    UNIQUE (account_id, post_id),

    -- Foreign key constraints
    FOREIGN KEY (account_id) REFERENCES accounts (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (post_id) REFERENCES posts (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE posts_mentions
(
    post_id      UUID NOT NULL,
    account_id   UUID NOT NULL,
    mention_text TEXT NOT NULL,
    PRIMARY KEY (post_id, account_id),

    -- Foreign key constraints
    FOREIGN KEY (post_id) REFERENCES posts (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (account_id) REFERENCES accounts (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE job_context
(
    id         UUID PRIMARY KEY,
    context    JSONB       NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE oauth2_applications
(
    id           UUID PRIMARY KEY,
    name         TEXT        NOT NULL,
    secret       TEXT        NOT NULL UNIQUE,
    scopes       TEXT        NOT NULL,
    redirect_uri TEXT        NOT NULL,
    website      TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE oauth2_authorization_codes
(
    code           TEXT PRIMARY KEY,
    application_id UUID        NOT NULL,
    user_id        UUID        NOT NULL,
    scopes         TEXT        NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Foreign key constraints
    FOREIGN KEY (application_id) REFERENCES oauth2_applications (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE oauth2_access_tokens
(
    token          TEXT PRIMARY KEY,
    user_id        UUID,
    application_id UUID,
    scopes         TEXT        NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at     TIMESTAMPTZ NOT NULL,

    -- Foreign key constraints
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (application_id) REFERENCES oauth2_applications (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE oauth2_refresh_tokens
(
    token          TEXT PRIMARY KEY,
    access_token   TEXT        NOT NULL UNIQUE,
    application_id UUID        NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Foreign key constraint
    FOREIGN KEY (access_token) REFERENCES oauth2_access_tokens (token) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (application_id) REFERENCES oauth2_applications (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE media_attachments
(
    id           UUID PRIMARY KEY,
    account_id   UUID,
    content_type TEXT        NOT NULL,
    description  TEXT,
    blurhash     TEXT,
    file_path    TEXT,
    remote_url   TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Foreign key constraints
    FOREIGN KEY (account_id) REFERENCES accounts (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE posts_media_attachments
(
    post_id             UUID NOT NULL,
    media_attachment_id UUID NOT NULL,
    PRIMARY KEY (post_id, media_attachment_id),

    -- Foreign key constraints
    FOREIGN KEY (post_id) REFERENCES posts (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (media_attachment_id) REFERENCES media_attachments (id) ON DELETE CASCADE ON UPDATE CASCADE
);

ALTER TABLE accounts
    ADD avatar_id UUID,
    ADD FOREIGN KEY (avatar_id) REFERENCES media_attachments (id)
        ON DELETE CASCADE
        ON UPDATE CASCADE;

ALTER TABLE accounts
    ADD header_id UUID,
    ADD FOREIGN KEY (header_id) REFERENCES media_attachments (id)
        ON DELETE CASCADE
        ON UPDATE CASCADE;

CREATE TABLE users_roles
(
    id         UUID PRIMARY KEY,
    user_id    UUID        NOT NULL,
    role       INTEGER     NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- UNIQUE constraints
    UNIQUE (user_id, role),

    -- Foreign key constraints
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE link_previews
(
    url        TEXT PRIMARY KEY,
    embed_data JSONB       NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE posts
    ADD COLUMN link_preview_url TEXT REFERENCES link_previews (url)
        ON DELETE SET NULL ON UPDATE CASCADE;

CREATE TABLE notifications
(
    id                    UUID PRIMARY KEY,
    receiving_account_id  UUID        NOT NULL,
    triggering_account_id UUID,
    post_id               UUID,
    notification_type     SMALLINT    NOT NULL,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- UNIQUE constraints
    UNIQUE (receiving_account_id, triggering_account_id, post_id, notification_type),

    -- Foreign key constraints
    FOREIGN KEY (receiving_account_id) REFERENCES accounts (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (triggering_account_id) REFERENCES accounts (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (post_id) REFERENCES posts (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE INDEX "idx-notifications-receiving_account_id" ON notifications (receiving_account_id);

CREATE TABLE custom_emojis
(
    id                  UUID PRIMARY KEY,
    shortcode           TEXT        NOT NULL,
    domain              TEXT,
    remote_id           TEXT        NOT NULL UNIQUE,
    media_attachment_id UUID        NOT NULL,
    endorsed            BOOLEAN     NOT NULL DEFAULT FALSE,

    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- UNIQUE constraints
    UNIQUE (shortcode, domain),

    -- Foreign key constraints
    FOREIGN KEY (media_attachment_id) REFERENCES media_attachments (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE posts_custom_emojis
(
    post_id         UUID NOT NULL,
    custom_emoji_id UUID NOT NULL,
    emoji_text      TEXT NOT NULL,
    PRIMARY KEY (post_id, custom_emoji_id),

    -- Foreign key constraints
    FOREIGN KEY (post_id) REFERENCES posts (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (custom_emoji_id) REFERENCES custom_emojis (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE INDEX "idx-custom_emojis-remote_id" ON custom_emojis (remote_id);
CREATE INDEX "idx-custom_emojis-shortcode" ON custom_emojis (shortcode);
CREATE INDEX "idx-custom_emojis-domain" ON custom_emojis (domain);

SELECT diesel_manage_updated_at('accounts');
SELECT diesel_manage_updated_at('accounts_follows');
SELECT diesel_manage_updated_at('posts');
SELECT diesel_manage_updated_at('users');
SELECT diesel_manage_updated_at('job_context');
SELECT diesel_manage_updated_at('oauth2_applications');
SELECT diesel_manage_updated_at('media_attachments');
SELECT diesel_manage_updated_at('link_previews');
SELECT diesel_manage_updated_at('custom_emojis');
