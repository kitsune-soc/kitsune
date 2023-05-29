DROP INDEX "idx-jobs-updated_at" ON jobs (updated_at);
DROP INDEX "idx-jobs-run_at" ON jobs (run_at);
DROP INDEX "idx-jobs-state" ON jobs (state);
DROP INDEX "idx-posts-visibility";
DROP INDEX "idx-posts-reposted_post_id";
DROP INDEX "idx-posts-in_reply_to_id";
DROP INDEX "idx-posts-account_id";
DROP INDEX "idx-accounts_follows-follower_id";
DROP INDEX "idx-accounts_follows-account_id";

DROP TABLE jobs;
DROP TABLE posts_mentions;
DROP TABLE posts_favourites;
DROP TABLE posts;
DROP TABLE users;
DROP TABLE accounts_follows;
DROP TABLE accounts;
