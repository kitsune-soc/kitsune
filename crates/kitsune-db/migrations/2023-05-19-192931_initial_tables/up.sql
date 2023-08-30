CREATE TABLE accounts (
    id UUID PRIMARY KEY,
    display_name TEXT,
    note TEXT,
    username TEXT NOT NULL,
    locked BOOLEAN NOT NULL,
    local BOOLEAN NOT NULL,
    domain TEXT NOT NULL,
    actor_type INTEGER NOT NULL,
    url TEXT UNIQUE NOT NULL,

    -- ActivityPub-specific data
    featured_collection_url TEXT,
    followers_url TEXT,
    following_url TEXT,
    inbox_url TEXT,
    outbox_url TEXT,
    shared_inbox_url TEXT,
    public_key_id TEXT NOT NULL UNIQUE,
    public_key TEXT NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Generated full-text search column
    account_ts TSVECTOR GENERATED ALWAYS AS (
        setweight(
            to_tsvector('simple', COALESCE(display_name, ''))
                || to_tsvector('simple', username),
            'A'
        )
            || setweight(to_tsvector('simple', COALESCE(note, '')), 'B')
    ) STORED NOT NULL,

    -- UNIQUE constraints
    UNIQUE (username, domain)
);

CREATE TABLE accounts_follows (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL,
    follower_id UUID NOT NULL,
    approved_at TIMESTAMPTZ,
    url TEXT NOT NULL UNIQUE,
    notify BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- UNIQUE constraints
    UNIQUE (account_id, follower_id),

    -- Foreign key constraints
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (follower_id) REFERENCES accounts(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE INDEX "idx-accounts_follows-account_id" ON accounts_follows (account_id);
CREATE INDEX "idx-accounts_follows-follower_id" ON accounts_follows (follower_id);

CREATE TABLE accounts_preferences (
    account_id UUID PRIMARY KEY,
    notify_on_follow BOOLEAN NOT NULL,
    notify_on_follow_request BOOLEAN NOT NULL,
    notify_on_repost BOOLEAN NOT NULL,
    notify_on_repost_update BOOLEAN NOT NULL,
    notify_on_favourite BOOLEAN NOT NULL,
    notify_on_mention BOOLEAN NOT NULL,

    -- Foreign key constraints
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE users (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL UNIQUE,
    oidc_id TEXT UNIQUE,
    username TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    password TEXT UNIQUE,
    domain TEXT NOT NULL,
    private_key TEXT NOT NULL,

     -- Email confirmation
    confirmed_at TIMESTAMPTZ,
    confirmation_token TEXT NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- UNIQUE constraints
    UNIQUE (username, domain),
    UNIQUE (confirmation_token),

    -- Foreign key constraints
    FOREIGN KEY (account_id) REFERENCES accounts(id)
);

-- This enum is automatically updated when starting Kitsune
-- It gets all the supported ISO codes pushed into it
CREATE TYPE language_iso_code AS ENUM ();

-- This is just a temporary function. This function is overwritten on each start-up of Kitsune using freshly read metadata
-- We need this for the migrations to succeed
CREATE FUNCTION iso_code_to_language (language_iso_code)
    RETURNS regconfig
    AS $$
        SELECT 'english'::regconfig
    $$
    LANGUAGE SQL IMMUTABLE;

CREATE TABLE posts (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL,

    in_reply_to_id UUID,
    reposted_post_id UUID,

    is_sensitive BOOLEAN NOT NULL,
    subject TEXT,

    content TEXT NOT NULL,
    content_source TEXT NOT NULL,
    content_lang language_iso_code NOT NULL,

    visibility INTEGER NOT NULL,
    is_local BOOLEAN NOT NULL,
    url TEXT NOT NULL UNIQUE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Generated full-text search column
    post_ts TSVECTOR GENERATED ALWAYS AS (
        to_tsvector(iso_code_to_language(content_lang), COALESCE(subject, ''))
            || to_tsvector(iso_code_to_language(content_lang), content)
    ) STORED NOT NULL,

    -- Foreign key constraints
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (in_reply_to_id) REFERENCES posts(id) ON DELETE SET NULL ON UPDATE CASCADE,
    FOREIGN KEY (reposted_post_id) REFERENCES posts(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE posts_favourites (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL,
    post_id UUID NOT NULL,
    url TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- UNIQUE constraints
    UNIQUE (account_id, post_id),

    -- Foreign key contraints
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE posts_mentions (
    post_id UUID NOT NULL,
    account_id UUID NOT NULL,
    mention_text TEXT NOT NULL,
    PRIMARY KEY (post_id, account_id),

    -- Foreign key constraints
    FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE INDEX "idx-posts-account_id" ON posts (account_id);
CREATE INDEX "idx-posts-in_reply_to_id" ON posts (in_reply_to_id);
CREATE INDEX "idx-posts-reposted_post_id" ON posts (reposted_post_id);
CREATE INDEX "idx-posts-visibility" ON posts (visibility);

-- Full-text search GIN indices
CREATE INDEX "idx-accounts-account_ts" ON accounts USING GIN (account_ts);
CREATE INDEX "idx-posts-post_ts" ON posts USING GIN (post_ts);

CREATE TABLE job_context (
    id UUID PRIMARY KEY,
    context JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

SELECT diesel_manage_updated_at('accounts');
SELECT diesel_manage_updated_at('accounts_follows');
SELECT diesel_manage_updated_at('posts');
SELECT diesel_manage_updated_at('users');
SELECT diesel_manage_updated_at('job_context');
