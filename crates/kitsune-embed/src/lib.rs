use diesel::{OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use http::{Method, Request};
use iso8601_timestamp::Timestamp;
use kitsune_db::{
    json::Json,
    model::link_preview::{ConflictLinkPreviewChangeset, LinkPreview, NewLinkPreview},
    schema::link_previews,
    with_connection, PgPool,
};
use kitsune_derive::kitsune_service;
use kitsune_error::Result;
use kitsune_http_client::Client as HttpClient;
use lantern_embed_sdk::EmbedWithExpire;
use schaber::Scraper;
use smol_str::SmolStr;
use std::{ops::ControlFlow, sync::LazyLock};

pub use lantern_embed_sdk::{Embed, EmbedType};

static LINK_SCRAPER: LazyLock<Scraper> = LazyLock::new(|| {
    Scraper::new("a:not(.mention, .hashtag)").expect("[Bug] Failed to parse link HTML selector")
});

fn first_link_from_fragment(fragment: &str) -> Option<String> {
    let mut link = None;
    LINK_SCRAPER
        .process(fragment, |element| {
            link = element.get_attribute("href");
            ControlFlow::Break(())
        })
        .unwrap();

    link
}

#[kitsune_service]
pub struct Client {
    db_pool: PgPool,
    #[builder(setter(into))]
    embed_service: SmolStr,
    #[builder(default)]
    http_client: HttpClient,
}

impl Client {
    /// Fetches embed data for an HTML fragment
    ///
    /// It parses the HTML fragment, selects the first link and fetched embed data for it
    pub async fn fetch_embed_for_fragment(
        &self,
        fragment: &str,
    ) -> Result<Option<LinkPreview<Embed>>> {
        let Some(url) = first_link_from_fragment(fragment) else {
            return Ok(None);
        };

        self.fetch_embed(&url).await.map(Some)
    }

    pub async fn fetch_embed(&self, url: &str) -> Result<LinkPreview<Embed>> {
        let embed_data = with_connection!(self.db_pool, |db_conn| {
            link_previews::table
                .find(url)
                .get_result::<LinkPreview<Embed>>(db_conn)
                .await
                .optional()
        })?;

        if let Some(data) = embed_data {
            if data.expires_at > Timestamp::now_utc() {
                return Ok(data);
            }
        }

        let request = Request::builder()
            .method(Method::POST)
            .uri(self.embed_service.as_str())
            .body(url.to_string().into())
            .unwrap();

        let response = HttpClient::execute(&self.http_client, request).await?;
        let (expires_at, embed_data): EmbedWithExpire = response.json().await?;

        let embed_data = with_connection!(self.db_pool, |db_conn| {
            diesel::insert_into(link_previews::table)
                .values(NewLinkPreview {
                    url,
                    embed_data: Json(&embed_data),
                    expires_at,
                })
                .on_conflict(link_previews::url)
                .do_update()
                .set(ConflictLinkPreviewChangeset {
                    embed_data: Json(&embed_data),
                    expires_at,
                })
                .get_result(db_conn)
                .await
        })?;

        Ok(embed_data)
    }
}
