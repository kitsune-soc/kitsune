use diesel::{OptionalExtension, QueryDsl};
use diesel_async::{pooled_connection::deadpool, RunQueryDsl};
use embed_sdk::Embed;
use http::{Method, Request};
use iso8601_timestamp::Timestamp;
use kitsune_db::{
    model::link_preview::{LinkPreview, NewLinkPreview},
    schema::link_previews,
    PgPool,
};
use kitsune_http_client::Client as HttpClient;
use once_cell::sync::Lazy;
use scraper::{Html, Selector};
use smol_str::SmolStr;
use typed_builder::TypedBuilder;

type Result<T, E = Error> = std::result::Result<T, E>;

static LINK_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("a").expect("[Bug] Failed to parse link HTML selector"));

fn first_link_from_fragment(fragment: &str) -> Option<String> {
    let parsed_fragment = Html::parse_fragment(fragment);

    parsed_fragment
        .select(&LINK_SELECTOR)
        .next()
        .and_then(|element| element.value().attr("href").map(ToString::to_string))
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),

    #[error(transparent)]
    Http(#[from] kitsune_http_client::Error),

    #[error(transparent)]
    Pool(#[from] deadpool::PoolError),
}

#[derive(Clone)]
pub struct FragmentEmbed {
    pub url: String,
    pub embed: Embed,
}

#[derive(Clone, TypedBuilder)]
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
    pub async fn fetch_embed_for_fragment(&self, fragment: &str) -> Result<Option<FragmentEmbed>> {
        let Some(url) = first_link_from_fragment(fragment) else {
            return Ok(None);
        };
        let embed = self.fetch_embed(&url).await?;

        Ok(Some(FragmentEmbed {
            url: url.to_string(),
            embed,
        }))
    }

    pub async fn fetch_embed(&self, url: &str) -> Result<Embed> {
        {
            let mut db_conn = self.db_pool.get().await?;
            if let Some(data) = link_previews::table
                .find(url)
                .get_result::<LinkPreview>(&mut db_conn)
                .await
                .optional()?
            {
                let embed_data = serde_json::from_value(data.embed_data)
                    .expect("[Bug] Invalid data in database");
                return Ok(embed_data);
            }
        }

        let request = Request::builder()
            .method(Method::POST)
            .uri(self.embed_service.as_str())
            .body(url.to_string().into())
            .unwrap();

        let response = HttpClient::execute(&self.http_client, request).await?;
        let (_expire, embed_data): (Timestamp, Embed) = response.json().await?;
        let embed_data_value = serde_json::to_value(embed_data.clone()).unwrap();

        let mut db_conn = self.db_pool.get().await?;
        diesel::insert_into(link_previews::table)
            .values(NewLinkPreview {
                url,
                embed_data: &embed_data_value,
            })
            .execute(&mut db_conn)
            .await?;

        Ok(embed_data)
    }
}
