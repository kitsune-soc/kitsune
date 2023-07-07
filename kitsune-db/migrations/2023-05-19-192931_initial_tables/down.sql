DROP INDEX "idx-posts-visibility";
DROP INDEX "idx-posts-reposted_post_id";
DROP INDEX "idx-posts-in_reply_to_id";
DROP INDEX "idx-posts-account_id";
DROP INDEX "idx-users-confirmation_token";
DROP INDEX "idx-accounts_follows-follower_id";
DROP INDEX "idx-accounts_follows-account_id";

DROP TABLE job_context;
DROP TABLE posts_mentions;
DROP TABLE posts_favourites;
DROP TABLE posts;
DROP TABLE users;
DROP TABLE accounts_follows;
DROP TABLE accounts;
