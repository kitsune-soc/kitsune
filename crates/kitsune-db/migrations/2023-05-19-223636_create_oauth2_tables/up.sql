CREATE TABLE oauth2_applications (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    secret TEXT NOT NULL UNIQUE,
    scopes TEXT NOT NULL,
    redirect_uri TEXT NOT NULL,
    website TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE oauth2_authorization_codes (
    code TEXT PRIMARY KEY,
    application_id UUID NOT NULL,
    user_id UUID NOT NULL,
    scopes TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Foreign key constraints
    FOREIGN KEY (application_id) REFERENCES oauth2_applications(id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE oauth2_access_tokens (
    token TEXT PRIMARY KEY,
    user_id UUID,
    application_id UUID,
    scopes TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,

    -- Foreign key constraints
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (application_id) REFERENCES oauth2_applications(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE oauth2_refresh_tokens (
    token TEXT PRIMARY KEY,
    access_token TEXT NOT NULL UNIQUE,
    application_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Foreign key constraint
    FOREIGN KEY (access_token) REFERENCES oauth2_access_tokens(token) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (application_id) REFERENCES oauth2_applications(id) ON DELETE CASCADE ON UPDATE CASCADE
);

SELECT diesel_manage_updated_at('oauth2_applications');
