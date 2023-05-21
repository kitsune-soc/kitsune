// @generated automatically by Diesel CLI.

pub mod sql_types {
    pub use diesel_full_text_search::Tsvector;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Tsvector;

    accounts (id) {
        id -> Uuid,
        display_name -> Nullable<Text>,
        note -> Nullable<Text>,
        username -> Text,
        locked -> Bool,
        local -> Bool,
        domain -> Text,
        actor_type -> Int4,
        url -> Nullable<Text>,
        featured_collection_url -> Nullable<Text>,
        followers_url -> Nullable<Text>,
        following_url -> Nullable<Text>,
        inbox_url -> Nullable<Text>,
        outbox_url -> Nullable<Text>,
        shared_inbox_url -> Nullable<Text>,
        public_key_id -> Text,
        public_key -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        display_name_ts -> Tsvector,
        note_ts -> Tsvector,
        username_ts -> Tsvector,
        avatar_id -> Nullable<Uuid>,
        header_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    accounts_follows (id) {
        id -> Uuid,
        account_id -> Uuid,
        follower_id -> Uuid,
        approved_at -> Nullable<Timestamptz>,
        url -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    jobs (id) {
        id -> Uuid,
        state -> Int4,
        context -> Jsonb,
        run_at -> Timestamptz,
        fail_count -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    media_attachments (id) {
        id -> Uuid,
        account_id -> Uuid,
        content_type -> Text,
        description -> Nullable<Text>,
        blurhash -> Nullable<Text>,
        file_path -> Nullable<Text>,
        remote_url -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    oauth2_access_tokens (token) {
        token -> Text,
        user_id -> Nullable<Uuid>,
        application_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        expired_at -> Timestamptz,
    }
}

diesel::table! {
    oauth2_applications (id) {
        id -> Uuid,
        name -> Text,
        secret -> Text,
        scopes -> Text,
        redirect_uri -> Text,
        website -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    oauth2_authorization_codes (code) {
        code -> Text,
        application_id -> Uuid,
        user_id -> Uuid,
        created_at -> Timestamptz,
        expired_at -> Timestamptz,
    }
}

diesel::table! {
    oauth2_refresh_tokens (token) {
        token -> Text,
        access_token -> Text,
        application_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Tsvector;

    posts (id) {
        id -> Uuid,
        account_id -> Uuid,
        in_reply_to_id -> Nullable<Uuid>,
        reposted_post_id -> Nullable<Uuid>,
        is_sensitive -> Bool,
        subject -> Nullable<Text>,
        content -> Text,
        visibility -> Int4,
        is_local -> Bool,
        url -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        subject_ts -> Tsvector,
        content_ts -> Tsvector,
    }
}

diesel::table! {
    posts_favourites (id) {
        id -> Uuid,
        account_id -> Uuid,
        post_id -> Uuid,
        url -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    posts_media_attachments (post_id, media_attachment_id) {
        post_id -> Uuid,
        media_attachment_id -> Uuid,
    }
}

diesel::table! {
    posts_mentions (post_id, account_id) {
        post_id -> Uuid,
        account_id -> Uuid,
        mention_text -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        account_id -> Uuid,
        oidc_id -> Nullable<Text>,
        username -> Text,
        email -> Text,
        password -> Nullable<Text>,
        domain -> Text,
        private_key -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    users_roles (id) {
        id -> Uuid,
        user_id -> Uuid,
        role -> Int4,
        created_at -> Timestamptz,
    }
}

diesel::joinable!(oauth2_access_tokens -> oauth2_applications (application_id));
diesel::joinable!(oauth2_access_tokens -> users (user_id));
diesel::joinable!(oauth2_authorization_codes -> oauth2_applications (application_id));
diesel::joinable!(oauth2_authorization_codes -> users (user_id));
diesel::joinable!(oauth2_refresh_tokens -> oauth2_access_tokens (access_token));
diesel::joinable!(oauth2_refresh_tokens -> oauth2_applications (application_id));
diesel::joinable!(posts -> accounts (account_id));
diesel::joinable!(posts_favourites -> accounts (account_id));
diesel::joinable!(posts_favourites -> posts (post_id));
diesel::joinable!(posts_media_attachments -> media_attachments (media_attachment_id));
diesel::joinable!(posts_media_attachments -> posts (post_id));
diesel::joinable!(posts_mentions -> accounts (account_id));
diesel::joinable!(posts_mentions -> posts (post_id));
diesel::joinable!(users -> accounts (account_id));
diesel::joinable!(users_roles -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    accounts,
    accounts_follows,
    jobs,
    media_attachments,
    oauth2_access_tokens,
    oauth2_applications,
    oauth2_authorization_codes,
    oauth2_refresh_tokens,
    posts,
    posts_favourites,
    posts_media_attachments,
    posts_mentions,
    users,
    users_roles,
);
