CREATE TABLE users_roles (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    role INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- UNIQUE constraints
    UNIQUE (user_id, role),

    -- Foreign key constraints
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE
);
