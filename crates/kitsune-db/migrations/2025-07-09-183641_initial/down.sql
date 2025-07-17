DROP TABLE posts_custom_emojis;
DROP TABLE custom_emojis;
DROP TABLE notifications;

ALTER TABLE posts
    DROP COLUMN link_preview_url;
DROP TABLE link_previews;

DROP TABLE users_roles;
DROP TABLE roles;

DROP TABLE posts_media_attachments;

ALTER TABLE accounts
    DROP COLUMN avatar_id;
ALTER TABLE accounts
    DROP COLUMN header_id;
DROP TABLE media_attachments;

DROP TABLE oauth2_refresh_tokens;
DROP TABLE oauth2_access_tokens;
DROP TABLE oauth2_authorization_codes;
DROP TABLE oauth2_applications;

DROP TABLE jobs;
DROP TYPE job_state;

DROP TABLE job_context;

DROP TABLE posts_mentions;
DROP TABLE posts_favourites;
DROP TABLE posts;

DROP TABLE users_accounts;

DROP TABLE accounts_preferences;
DROP TABLE accounts_follows;
DROP TABLE accounts_activitypub;
DROP TABLE accounts_cryptographic_keys;
DROP TABLE cryptographic_keys;
DROP TABLE accounts;

DROP TABLE domains;
DROP TABLE users;

DROP SCHEMA kitsune CASCADE;
