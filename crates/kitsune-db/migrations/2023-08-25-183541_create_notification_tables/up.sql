CREATE TABLE notifications (
    id UUID PRIMARY KEY,
    receiving_account_id UUID NOT NULL,
    triggering_account_id UUID,
    post_id UUID,
    notification_type SMALLINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- UNIQUE constraints
    UNIQUE (receiving_account_id, triggering_account_id, post_id, notification_type),

    -- Foreign key constraints
    FOREIGN KEY (receiving_account_id) REFERENCES accounts(id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (triggering_account_id) REFERENCES accounts(id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE INDEX "idx-notifications-receiving_account_id" ON notifications (receiving_account_id);
