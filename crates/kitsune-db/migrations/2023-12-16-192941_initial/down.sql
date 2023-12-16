DROP INDEX "idx-custom_emojis-remote_id";
DROP INDEX "idx-custom_emojis-shortcode";
DROP INDEX "idx-custom_emojis-domain";

DROP TABLE posts_custom_emojis;
DROP TABLE custom_emojis;

DROP INDEX "idx-notifications-receiving_account_id";

DROP TABLE notifications;

ALTER TABLE posts DROP COLUMN link_preview_url;
DROP TABLE link_previews;

DROP TABLE users_roles;

ALTER TABLE accounts DROP COLUMN header_id;
ALTER TABLE accounts DROP COLUMN avatar_id;

DROP TABLE posts_media_attachments;
DROP TABLE media_attachments;

DROP TABLE oauth2_refresh_tokens;
DROP TABLE oauth2_access_tokens;
DROP TABLE oauth2_authorization_codes;
DROP TABLE oauth2_applications;

DROP INDEX "idx-posts-post_ts";
DROP INDEX "idx-accounts-account_ts";

DROP INDEX "idx-posts-visibility";
DROP INDEX "idx-posts-reposted_post_id";
DROP INDEX "idx-posts-in_reply_to_id";
DROP INDEX "idx-posts-account_id";
DROP INDEX "idx-accounts_follows-follower_id";
DROP INDEX "idx-accounts_follows-account_id";

DROP TABLE job_context;
DROP TABLE posts_mentions;
DROP TABLE posts_favourites;

DROP TABLE posts;
DROP FUNCTION iso_code_to_language;
DROP TYPE language_iso_code;

DROP TABLE users;
DROP TABLE accounts_preferences;
DROP TABLE accounts_follows;
DROP TABLE accounts;
