use crate::http::handler::{mastodon, nodeinfo, well_known};
use kitsune_type::{
    mastodon as mastodon_type, nodeinfo as nodeinfo_type, webfinger as webfinger_type,
};
use utoipa::{
    openapi::security::{AuthorizationCode, Flow, OAuth2, Scopes, SecurityScheme},
    Modify, OpenApi, ToSchema,
};

#[derive(ToSchema)]
pub struct MediaAttachmentBody {
    pub description: Option<String>,
    #[schema(value_type = String, format = Binary)]
    pub file: (),
}

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "oauth_token",
                SecurityScheme::OAuth2(OAuth2::new([Flow::AuthorizationCode(
                    AuthorizationCode::new("/oauth/authorize", "/oauth/token", Scopes::new()),
                )])),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    components(schemas(
        MediaAttachmentBody,
        mastodon::api::v1::apps::AppForm,
        mastodon::api::v1::media::UpdateAttachment,
        mastodon::api::v1::statuses::CreateForm,
        mastodon::api::v2::search::SearchType,
        mastodon_type::App,
        mastodon_type::account::Account,
        mastodon_type::account::Field,
        mastodon_type::account::Source,
        mastodon_type::instance::Stats,
        mastodon_type::instance::Urls,
        mastodon_type::instance::Instance,
        mastodon_type::media_attachment::MediaType,
        mastodon_type::media_attachment::MediaAttachment,
        mastodon_type::relationship::Relationship,
        mastodon_type::search::SearchResult,
        mastodon_type::status::Context,
        mastodon_type::status::Mention,
        mastodon_type::status::Visibility,
        mastodon_type::status::Status,
        nodeinfo_type::two_one::TwoOne,
        nodeinfo_type::two_one::Protocol,
        nodeinfo_type::two_one::InboundService,
        nodeinfo_type::two_one::OutboundService,
        nodeinfo_type::two_one::Version,
        nodeinfo_type::two_one::Software,
        nodeinfo_type::two_one::Services,
        nodeinfo_type::two_one::UsageUsers,
        nodeinfo_type::two_one::Usage,
        nodeinfo_type::well_known::Rel,
        nodeinfo_type::well_known::Link,
        nodeinfo_type::well_known::WellKnown,
        webfinger_type::Link,
        webfinger_type::Resource,
    )),
    modifiers(&SecurityAddon),
    paths(
        mastodon::api::v1::accounts::get,
        mastodon::api::v1::accounts::relationships::get,
        mastodon::api::v1::accounts::statuses::get,
        mastodon::api::v1::accounts::verify_credentials::get,
        mastodon::api::v1::apps::post,
        mastodon::api::v1::instance::get,
        mastodon::api::v1::media::post,
        mastodon::api::v1::media::put,
        mastodon::api::v1::statuses::delete,
        mastodon::api::v1::statuses::get,
        mastodon::api::v1::statuses::post,
        mastodon::api::v1::statuses::context::get,
        mastodon::api::v1::statuses::favourite::post,
        mastodon::api::v1::statuses::unfavourite::post,
        mastodon::api::v1::timelines::home::get,
        mastodon::api::v1::timelines::public::get,
        mastodon::api::v2::search::get,
        nodeinfo::two_one::get,
        well_known::nodeinfo::get,
        well_known::webfinger::get,
    ),
)]
pub struct ApiDocs;
