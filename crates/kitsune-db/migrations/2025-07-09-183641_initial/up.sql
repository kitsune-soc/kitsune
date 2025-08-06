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

--
-- Now follow the actual table creation routines
--

-- BEGIN "users" TABLE

CREATE TABLE users
(
    id                 UUID PRIMARY KEY,
    oidc_id            TEXT,

    -- Use special collation to ignore case and accent differences
    username           TEXT        NOT NULL COLLATE kitsune.ignore_accent_case,
    email              TEXT        NOT NULL,
    password           TEXT,

    -- Email confirmation
    confirmed_at       TIMESTAMPTZ,
    confirmation_token TEXT        NOT NULL,

    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- UNIQUE constraints
ALTER TABLE users
    ADD CONSTRAINT "uk-users-oidc_id"
        UNIQUE (oidc_id);

ALTER TABLE users
    ADD CONSTRAINT "uk-users-username"
        UNIQUE (username);

ALTER TABLE users
    ADD CONSTRAINT "uk-users-email"
        UNIQUE (email);

ALTER TABLE users
    ADD CONSTRAINT "uk-users-password"
        UNIQUE (password);

ALTER TABLE users
    ADD CONSTRAINT "uk-users-confirmation_token"
        UNIQUE (confirmation_token);

-- Register triggers

SELECT diesel_manage_updated_at('users');

-- END "users" TABLE

-- BEGIN "domains" TABLE

CREATE TABLE domains
(
    domain             TEXT PRIMARY KEY,
    owner_id           UUID REFERENCES users (id),
    challenge_value    TEXT,
    globally_available BOOLEAN     NOT NULL DEFAULT FALSE,
    verified_at        TIMESTAMPTZ,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE domains
    ADD CONSTRAINT "fk-domains-owner_id"
        FOREIGN KEY (owner_id) REFERENCES users (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE;

-- Register triggers

SELECT diesel_manage_updated_at('domains');

-- END "domains" TABLE

-- BEGIN "accounts" TABLE

CREATE TABLE accounts
(
    id           UUID PRIMARY KEY,

    account_type INTEGER                                                  NOT NULL,

    avatar_id    UUID,
    header_id    UUID,
    display_name TEXT,
    note         TEXT,

    -- Use special collation to ignore case and accent differences
    username     TEXT                                                     NOT NULL COLLATE kitsune.ignore_accent_case,

    locked       BOOLEAN                                                  NOT NULL,
    local        BOOLEAN                                                  NOT NULL,
    domain       TEXT                                                     NOT NULL,
    url          TEXT                                                     NOT NULL,

    created_at   TIMESTAMPTZ                                              NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ                                              NOT NULL DEFAULT NOW(),

    -- Generated full-text search column
    account_ts   TSVECTOR GENERATED ALWAYS AS (
        setweight(to_tsvector('simple', COALESCE(display_name, '')) ||
                  to_tsvector('simple', username), 'A') ||
        setweight(to_tsvector('simple', COALESCE(note, '')), 'B')) STORED NOT NULL
);

-- UNIQUE constraints
ALTER TABLE accounts
    ADD CONSTRAINT "uk-accounts-url"
        UNIQUE (url);

ALTER TABLE accounts
    ADD CONSTRAINT "uk-accounts-username-domain"
        UNIQUE (username, domain);

-- Foreign key constraints
--ALTER TABLE accounts
--    ADD CONSTRAINT "uk-accounts-domains-domain"
--        FOREIGN KEY (domain) REFERENCES domains (domain)
--            ON DELETE SET NULL
--            ON UPDATE CASCADE;

-- Create indexes

CREATE INDEX "idx-accounts-account_ts" ON accounts USING GIN (account_ts);

-- Register triggers

SELECT diesel_manage_updated_at('accounts');

-- END "accounts" TABLE

-- BEGIN "cryptographic_keys" TABLE

CREATE TABLE cryptographic_keys
(
    key_id          TEXT PRIMARY KEY,
    public_key_der  BYTEA       NOT NULL,
    private_key_der BYTEA,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- END "cryptographic_keys" TABLE

-- BEGIN "accounts_cryptographic_keys" TABLE

CREATE TABLE accounts_cryptographic_keys
(
    account_id UUID,
    key_id     TEXT,
    PRIMARY KEY (account_id, key_id)
);

ALTER TABLE accounts_cryptographic_keys
    ADD CONSTRAINT "fk-accounts_cryptographic_keys-account_id"
        FOREIGN KEY (account_id) REFERENCES accounts (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE;

ALTER TABLE accounts_cryptographic_keys
    ADD CONSTRAINT "fk-accounts_cryptographic_keys-key_id"
        FOREIGN KEY (key_id) REFERENCES cryptographic_keys (key_id)
            ON DELETE CASCADE
            ON UPDATE CASCADE;

-- END "accounts_cryptographic_keys" TABLE

-- BEGIN "accounts_activitypub" TABLE

CREATE TABLE accounts_activitypub
(
    account_id              UUID PRIMARY KEY,
    featured_collection_url TEXT,
    followers_url           TEXT,
    following_url           TEXT,
    inbox_url               TEXT,
    outbox_url              TEXT,
    shared_inbox_url        TEXT,
    key_id                  TEXT NOT NULL
);

ALTER TABLE accounts_activitypub
    ADD CONSTRAINT "fk-accounts_activitypub-account_id"
        FOREIGN KEY (account_id) REFERENCES accounts (id) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE accounts_activitypub
    ADD CONSTRAINT "fk-accounts_activitypub-key_id"
        FOREIGN KEY (key_id) REFERENCES cryptographic_keys (key_id) ON DELETE CASCADE ON UPDATE CASCADE;

-- END "accounts_activitypub" TABLE

-- BEGIN "accounts_follows" TABLE

CREATE TABLE accounts_follows
(
    id          UUID PRIMARY KEY,
    account_id  UUID        NOT NULL,
    follower_id UUID        NOT NULL,
    approved_at TIMESTAMPTZ,
    url         TEXT        NOT NULL,
    notify      BOOLEAN     NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- UNIQUE constraints
ALTER TABLE accounts_follows
    ADD CONSTRAINT "uk-accounts_follows-url"
        UNIQUE (url);

ALTER TABLE accounts_follows
    ADD CONSTRAINT "uk-accounts_follows-account_id-follower_id"
        UNIQUE (account_id, follower_id);

-- Foreign key constraints
ALTER TABLE accounts_follows
    ADD CONSTRAINT "const-foreign-accounts_follows-account_id"
        FOREIGN KEY (account_id) REFERENCES accounts (id) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE accounts_follows
    ADD CONSTRAINT "fk-accounts_follows-follower_id"
        FOREIGN KEY (follower_id) REFERENCES accounts (id) ON DELETE CASCADE ON UPDATE CASCADE;

-- Create indexes

CREATE INDEX "idx-accounts_follows-account_id" ON accounts_follows (account_id);
CREATE INDEX "idx-accounts_follows-follower_id" ON accounts_follows (follower_id);

-- Register triggers

SELECT diesel_manage_updated_at('accounts_follows');

-- END "accounts_follows" TABLE

-- BEGIN "accounts_preferences" TABLE

CREATE TABLE accounts_preferences
(
    account_id                      UUID       PRIMARY KEY,
    
    notify_on_follow                BOOLEAN    NOT NULL,
    notify_on_follow_request        BOOLEAN    NOT NULL,
    notify_on_repost                BOOLEAN    NOT NULL,
    notify_on_post_update           BOOLEAN    NOT NULL,
    notify_on_favourite             BOOLEAN    NOT NULL,
    notify_on_mention               BOOLEAN    NOT NULL
);

-- Foreign key constraints
ALTER TABLE accounts_preferences
    ADD CONSTRAINT "fk-accounts_preferences-account_id"
        FOREIGN KEY (account_id)
            REFERENCES accounts (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE;

-- END "accounts_preferences" TABLE

-- BEGIN "users_accounts" TABLE

CREATE TABLE users_accounts
(
    user_id    UUID,
    account_id UUID,
    PRIMARY KEY (user_id, account_id)
);

-- Foreign key constraints
ALTER TABLE users_accounts
    ADD CONSTRAINT "fk-users_accounts-user_id"
        FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE users_accounts
    ADD CONSTRAINT "fk-users_accounts-account_id"
        FOREIGN KEY (account_id) REFERENCES accounts (id) ON DELETE CASCADE ON UPDATE CASCADE;

-- END "users_accounts" TABLE

-- BEGIN "link_previews" TABLE

CREATE TABLE link_previews
(
    url        TEXT PRIMARY KEY,
    embed_data JSONB       NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Register triggers

SELECT diesel_manage_updated_at('link_previews');

-- END "link_previews" TABLE

-- BEGIN "posts" TABLE

CREATE TABLE posts
(
    id               UUID PRIMARY KEY,
    account_id       UUID                                                                                                   NOT NULL,

    in_reply_to_id   UUID,
    reposted_post_id UUID,

    subject          TEXT,

    content          TEXT                                                                                                   NOT NULL,
    content_source   TEXT                                                                                                   NOT NULL,
    content_lang     kitsune.language_iso_code                                                                              NOT NULL,
    link_preview_url TEXT,

    visibility       INTEGER                                                                                                NOT NULL,
    is_local         BOOLEAN                                                                                                NOT NULL,
    url              TEXT                                                                                                   NOT NULL,

    created_at       TIMESTAMPTZ                                                                                            NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ                                                                                            NOT NULL DEFAULT NOW(),

    -- Generated full-text search column
    post_ts          TSVECTOR GENERATED ALWAYS AS (to_tsvector(kitsune.iso_code_to_language(content_lang),
                                                               COALESCE(subject, '')) ||
                                                   to_tsvector(kitsune.iso_code_to_language(content_lang), content)) STORED NOT NULL
);

-- UNIQUE constraints
ALTER TABLE posts
    ADD CONSTRAINT "uk-posts-url"
        UNIQUE (url);

-- Foreign key constraints
ALTER TABLE posts
    ADD CONSTRAINT "fk-posts-account_id"
        FOREIGN KEY (account_id) REFERENCES accounts (id) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE posts
    ADD CONSTRAINT "fk-posts-in_reply_to_id"
        FOREIGN KEY (in_reply_to_id) REFERENCES posts (id) ON DELETE SET NULL ON UPDATE CASCADE;

ALTER TABLE posts
    ADD CONSTRAINT "fk-posts-reposted_post_id"
        FOREIGN KEY (reposted_post_id) REFERENCES posts (id) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE posts
    ADD CONSTRAINT "fk-posts-link_preview_url"
        FOREIGN KEY (link_preview_url) REFERENCES link_previews (url)
            ON DELETE SET NULL
            ON UPDATE CASCADE;

-- Create indexes

CREATE INDEX "idx-posts-account_id" ON posts (account_id);
CREATE INDEX "idx-posts-in_reply_to_id" ON posts (in_reply_to_id);
CREATE INDEX "idx-posts-reposted_post_id" ON posts (reposted_post_id);
CREATE INDEX "idx-posts-visibility" ON posts (visibility);
CREATE INDEX "idx-posts-post_ts" ON posts USING GIN (post_ts);

-- Register triggers

SELECT diesel_manage_updated_at('posts');

-- END "posts" TABLE

-- BEGIN "media_attachments" TABLE

CREATE TABLE media_attachments
(
    id           UUID PRIMARY KEY,
    account_id   UUID,
    content_type TEXT        NOT NULL,
    description  TEXT,
    file_path    TEXT,
    is_sensitive BOOLEAN     NOT NULL,
    remote_url   TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Foreign key constraints
ALTER TABLE media_attachments
    ADD CONSTRAINT "fk-media_attachments-account_id"
        FOREIGN KEY (account_id) REFERENCES accounts (id) ON DELETE CASCADE ON UPDATE CASCADE;

-- Add columns for avatars and headers to the "accounts" table
ALTER TABLE accounts
    ADD CONSTRAINT "fk-accounts-avatar_id"
        FOREIGN KEY (avatar_id) REFERENCES media_attachments (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE;

ALTER TABLE accounts
    ADD CONSTRAINT "fk-accounts-header_id"
        FOREIGN KEY (header_id) REFERENCES media_attachments (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE;

-- Register triggers

SELECT diesel_manage_updated_at('media_attachments');

-- END "media_attachments" TABLE

-- BEGIN "posts_media_attachments" TABLE

CREATE TABLE posts_media_attachments
(
    post_id             UUID NOT NULL,
    media_attachment_id UUID NOT NULL,
    PRIMARY KEY (post_id, media_attachment_id)
);

-- Foreign key constraints
ALTER TABLE posts_media_attachments
    ADD CONSTRAINT "fk-posts_media_attachments-post_id"
        FOREIGN KEY (post_id) REFERENCES posts (id) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE posts_media_attachments
    ADD CONSTRAINT "fk-posts_media_attachments-media_attachment_id"
        FOREIGN KEY (media_attachment_id) REFERENCES media_attachments (id) ON DELETE CASCADE ON UPDATE CASCADE;

-- END "posts_media_attachments" TABLE

-- BEGIN "posts_favourites" TABLE

CREATE TABLE posts_favourites
(
    id         UUID PRIMARY KEY,
    account_id UUID        NOT NULL,
    post_id    UUID        NOT NULL,
    url        TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- UNIQUE constraints
ALTER TABLE posts_favourites
    ADD CONSTRAINT "uk-posts_favourites-url"
        UNIQUE (url);

ALTER TABLE posts_favourites
    ADD CONSTRAINT "uk-posts_favourites-account_id-post_id"
        UNIQUE (account_id, post_id);

-- Foreign key constraints
ALTER TABLE posts_favourites
    ADD CONSTRAINT "fk-posts_favourites-account_id"
        FOREIGN KEY (account_id) REFERENCES accounts (id) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE posts_favourites
    ADD CONSTRAINT "fk-posts_favourites-post_id"
        FOREIGN KEY (post_id) REFERENCES posts (id) ON DELETE CASCADE ON UPDATE CASCADE;

-- END "posts_favourites" TABLE

-- BEGIN "posts_mentions" TABLE

CREATE TABLE posts_mentions
(
    post_id      UUID NOT NULL,
    account_id   UUID NOT NULL,
    mention_text TEXT NOT NULL,
    PRIMARY KEY (post_id, account_id)
);

-- Foreign key constraints
ALTER TABLE posts_mentions
    ADD CONSTRAINT "fk-posts_mentions-post_id"
        FOREIGN KEY (post_id) REFERENCES posts (id) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE posts_mentions
    ADD CONSTRAINT "fk-posts_mentions-account_id"
        FOREIGN KEY (account_id) REFERENCES accounts (id) ON DELETE CASCADE ON UPDATE CASCADE;

-- END "posts_mentions" TABLE

-- BEGIN "job_context" TABLE

CREATE TABLE job_context
(
    id         UUID PRIMARY KEY,
    context    JSONB       NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Register triggers

SELECT diesel_manage_updated_at('job_context');

-- END "job_context" TABLE

-- BEGIN "jobs" TABLE

CREATE TYPE job_state as ENUM (
    'queued',
    'running',
    'failed',
    'completed'
);

CREATE TABLE jobs (
    id UUID PRIMARY KEY,
    meta JSONB NOT NULL,

    state job_state NOT NULL,
    fail_count INTEGER NOT NULL DEFAULT 0,
    run_at TIMESTAMPTZ NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Foreign key constraints

ALTER TABLE jobs
    ADD CONSTRAINT "fk-jobs-id"
        FOREIGN KEY (id) REFERENCES job_context (id) ON DELETE CASCADE ON UPDATE CASCADE;

-- Create indexes

CREATE INDEX "idx-jobs-run_at" ON jobs (run_at);
CREATE INDEX "idx-jobs-state" ON jobs USING HASH (state);
CREATE INDEX "idx-jobs-updated_at" ON jobs (updated_at);

-- Register triggers

SELECT diesel_manage_updated_at('jobs');

-- END "jobs" TABLE

-- BEGIN "oauth2_applications" TABLE

CREATE TABLE oauth2_applications
(
    id           UUID PRIMARY KEY,
    name         TEXT        NOT NULL,
    secret       TEXT        NOT NULL,
    scopes       TEXT        NOT NULL,
    redirect_uri TEXT        NOT NULL,
    website      TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- UNIQUE constraints
ALTER TABLE oauth2_applications
    ADD CONSTRAINT "uk-oauth2_applications-secret"
        UNIQUE (secret);

-- Register triggers

SELECT diesel_manage_updated_at('oauth2_applications');

-- END "oauth2_applications" TABLE

-- BEGIN "oauth2_authorization_codes" TABLE

CREATE TABLE oauth2_authorization_codes
(
    code           TEXT PRIMARY KEY,
    application_id UUID        NOT NULL,
    user_id        UUID        NOT NULL,
    scopes         TEXT        NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Foreign key constraints
ALTER TABLE oauth2_authorization_codes
    ADD CONSTRAINT "fk-oauth2_authorization_codes-application_id"
        FOREIGN KEY (application_id) REFERENCES oauth2_applications (id) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE oauth2_authorization_codes
    ADD CONSTRAINT "fk-oauth2_authorization_codes-user_id"
        FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE ON UPDATE CASCADE;

-- Register triggers

SELECT diesel_manage_updated_at('oauth2_authorization_codes');

-- END "oauth2_authorization_codes" TABLE

-- BEGIN "oauth2_access_tokens" TABLE

CREATE TABLE oauth2_access_tokens
(
    token          TEXT PRIMARY KEY,
    user_id        UUID,
    application_id UUID,
    scopes         TEXT        NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at     TIMESTAMPTZ NOT NULL
);

-- Foreign key constraints
ALTER TABLE oauth2_access_tokens
    ADD CONSTRAINT "fk-oauth2_access_tokens-user_id"
        FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE oauth2_access_tokens
    ADD CONSTRAINT "fk-oauth2_access_tokens-application_id"
        FOREIGN KEY (application_id) REFERENCES oauth2_applications (id) ON DELETE CASCADE ON UPDATE CASCADE;

-- Register triggers

SELECT diesel_manage_updated_at('oauth2_access_tokens');

-- END "oauth2_access_tokens" TABLE

-- BEGIN "oauth2_refresh_tokens" TABLE

CREATE TABLE oauth2_refresh_tokens
(
    token          TEXT PRIMARY KEY,
    access_token   TEXT        NOT NULL,
    application_id UUID        NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- UNIQUE constraints
ALTER TABLE oauth2_refresh_tokens
    ADD CONSTRAINT "uk-oauth2_refresh_tokens-access_token"
        UNIQUE (access_token);

-- Foreign key constraint
ALTER TABLE oauth2_refresh_tokens
    ADD CONSTRAINT "fk-oauth2_refresh_tokens-access_token"
        FOREIGN KEY (access_token) REFERENCES oauth2_access_tokens (token) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE oauth2_refresh_tokens
    ADD CONSTRAINT "fk-oauth2_refresh_tokens-application_id"
        FOREIGN KEY (application_id) REFERENCES oauth2_applications (id) ON DELETE CASCADE ON UPDATE CASCADE;

-- Register triggers

SELECT diesel_manage_updated_at('oauth2_refresh_tokens');

-- END "oauth2_refresh_tokens" TABLE

-- BEGIN "roles" TABLE

CREATE TABLE roles
(
    id           UUID PRIMARY KEY,
    name         TEXT          NOT NULL,
    capabilities INTEGER ARRAY NOT NULL,
    created_at   TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ   NOT NULL DEFAULT NOW()
);

-- UNIQUE constraints

ALTER TABLE roles
    ADD CONSTRAINT "uk-roles-name"
        UNIQUE (name);

-- Register triggers

SELECT diesel_manage_updated_at('roles');

-- END "roles" TABLE

-- BEGIN "users_roles" TABLE

CREATE TABLE users_roles
(
    user_id UUID NOT NULL,
    role_id UUID NOT NULL,
    PRIMARY KEY (user_id, role_id)
);

-- Foreign key constraints
ALTER TABLE users_roles
    ADD CONSTRAINT "fk-users_roles-user_id"
        FOREIGN KEY (user_id) REFERENCES users (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE;

ALTER TABLE users_roles
    ADD CONSTRAINT "fk-users_roles-role_id"
        FOREIGN KEY (role_id) REFERENCES roles (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE;

-- END "users_roles" TABLE

-- BEGIN "notifications" TABLE

CREATE TABLE notifications
(
    id                    UUID PRIMARY KEY,
    receiving_account_id  UUID        NOT NULL,
    triggering_account_id UUID,
    post_id               UUID,
    notification_type     SMALLINT    NOT NULL,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- UNIQUE constraints
ALTER TABLE notifications
    ADD CONSTRAINT "uk-notifications-ra_id-tr_id-post_id-notification_ty"
        UNIQUE (receiving_account_id, triggering_account_id, post_id, notification_type);

-- Foreign key constraints
ALTER TABLE notifications
    ADD CONSTRAINT "fk-notifications-receiving_account_id"
        FOREIGN KEY (receiving_account_id) REFERENCES accounts (id) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE notifications
    ADD CONSTRAINT "fk-notifications-triggering_account_id"
        FOREIGN KEY (triggering_account_id) REFERENCES accounts (id) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE notifications
    ADD CONSTRAINT "fk-notifications-post_id"
        FOREIGN KEY (post_id) REFERENCES posts (id) ON DELETE CASCADE ON UPDATE CASCADE;

-- Create indexes

CREATE INDEX "idx-notifications-receiving_account_id" ON notifications (receiving_account_id);

-- END "notifications" table

-- CREATE "custom_emojis" table

CREATE TABLE custom_emojis
(
    id                  UUID PRIMARY KEY,
    shortcode           TEXT        NOT NULL,
    domain              TEXT,
    remote_id           TEXT        NOT NULL,
    media_attachment_id UUID        NOT NULL,
    endorsed            BOOLEAN     NOT NULL DEFAULT FALSE,

    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- UNIQUE constraints
ALTER TABLE custom_emojis
    ADD CONSTRAINT "uk-custom_emojis-remote_id"
        UNIQUE (remote_id);

ALTER TABLE custom_emojis
    ADD CONSTRAINT "uk-custom_emojis-shortcode-domain"
        UNIQUE (shortcode, domain);

-- Foreign key constraints
ALTER TABLE custom_emojis
    ADD CONSTRAINT "fk-custom_emojis-media_attachment_id"
        FOREIGN KEY (media_attachment_id) REFERENCES media_attachments (id) ON DELETE CASCADE ON UPDATE CASCADE;

-- Register triggers

SELECT diesel_manage_updated_at('custom_emojis');

-- END "custom_emojis" TABLE

-- CREATE "posts_custom_emojis" TABLE

CREATE TABLE posts_custom_emojis
(
    post_id         UUID NOT NULL,
    custom_emoji_id UUID NOT NULL,
    emoji_text      TEXT NOT NULL,
    PRIMARY KEY (post_id, custom_emoji_id)
);

-- Foreign key constraints
ALTER TABLE posts_custom_emojis
    ADD CONSTRAINT "fk-posts_custom_emojis-post_id"
        FOREIGN KEY (post_id) REFERENCES posts (id) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE posts_custom_emojis
    ADD CONSTRAINT "fk-posts_custom_emojis-custom_emoji_id"
        FOREIGN KEY (custom_emoji_id) REFERENCES custom_emojis (id) ON DELETE CASCADE ON UPDATE CASCADE;

-- Create indexes

CREATE INDEX "idx-custom_emojis-remote_id" ON custom_emojis (remote_id);
CREATE INDEX "idx-custom_emojis-shortcode" ON custom_emojis (shortcode);
CREATE INDEX "idx-custom_emojis-domain" ON custom_emojis (domain);

-- END "custom_emojis" TABLE
