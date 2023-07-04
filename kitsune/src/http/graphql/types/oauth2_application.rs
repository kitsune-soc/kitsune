use async_graphql::SimpleObject;
use kitsune_db::model::oauth2;
use speedy_uuid::Uuid;
use time::OffsetDateTime;

#[derive(Clone, Debug, Eq, PartialEq, SimpleObject)]
#[graphql(name = "OAuth2Application")]
pub struct OAuth2Application {
    pub id: Uuid,
    pub name: String,
    pub secret: String,
    pub redirect_uri: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl From<oauth2::Application> for OAuth2Application {
    fn from(value: oauth2::Application) -> Self {
        Self {
            id: value.id,
            name: value.name,
            secret: value.secret,
            redirect_uri: value.redirect_uri,
            created_at: value.created_at.assume_utc(),
            updated_at: value.updated_at.assume_utc(),
        }
    }
}
