use crate::{
    db::entity::{post, user},
    error::{Error, Result},
    util::CleanHtmlExt,
};
use chrono::Utc;
use http::{HeaderMap, HeaderValue};
use phenomenon_model::ap::object::{Actor, Note};
use reqwest::Client;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
};
use url::Url;
use uuid::Uuid;

#[derive(Clone)]
pub struct Fetcher {
    client: Client,
    db_conn: DatabaseConnection,
}

impl Fetcher {
    pub fn new(db_conn: DatabaseConnection) -> Self {
        let mut default_headers = HeaderMap::new();
        default_headers.insert(
            "Accept",
            HeaderValue::from_static("application/activity+json"),
        );

        Self {
            client: Client::builder()
                .default_headers(default_headers)
                .user_agent(concat!(
                    env!("CARGO_PKG_NAME"),
                    "/",
                    env!("CARGO_PKG_VERSION"),
                ))
                .build()
                .unwrap(),
            db_conn,
        }
    }

    pub async fn fetch_actor(&self, url: &str) -> Result<user::Model> {
        if let Some(user) = user::Entity::find()
            .filter(user::Column::Url.eq(url))
            .one(&self.db_conn)
            .await?
        {
            return Ok(user);
        }

        let url = Url::parse(url)?;
        let mut actor: Actor = self.client.get(url.clone()).send().await?.json().await?;
        actor.clean_html();

        user::Model {
            id: Uuid::new_v4(),
            // TODO: Push in URLs from the actors
            avatar: None,
            header: None,
            display_name: actor.name,
            note: actor.subject,
            username: actor.preferred_username,
            email: None,
            password: None,
            domain: Some(url.host_str().unwrap().into()),
            url: actor.rest.id,
            inbox_url: actor.inbox,
            public_key: Some(actor.public_key.public_key_pem),
            private_key: None,
            created_at: actor.rest.published_at,
            updated_at: Utc::now(),
        }
        .into_active_model()
        .insert(&self.db_conn)
        .await
        .map_err(Error::from)
    }

    pub async fn fetch_note(&self, url: &str) -> Result<post::Model> {
        if let Some(post) = post::Entity::find()
            .filter(post::Column::Url.eq(url))
            .one(&self.db_conn)
            .await?
        {
            return Ok(post);
        }

        let mut note: Note = self.client.get(url).send().await?.json().await?;
        note.clean_html();

        let user = self
            .fetch_actor(note.rest.attributed_to().ok_or(Error::MalformedApObject)?)
            .await?;

        post::Model {
            id: Uuid::new_v4(),
            user_id: user.id,
            subject: note.subject,
            content: note.content,
            url: note.rest.id,
            created_at: note.rest.published_at,
            updated_at: Utc::now(),
        }
        .into_active_model()
        .insert(&self.db_conn)
        .await
        .map_err(Error::from)
    }
}
