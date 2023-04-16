use async_graphql::SimpleObject;
use kitsune_db::entity::oauth2_applications;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, SimpleObject)]
#[graphql(name = "OAuth2Application")]
pub struct Oauth2Application {
    pub id: Uuid,
    pub name: String,
    pub secret: String,
    pub redirect_uri: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl From<oauth2_applications::Model> for Oauth2Application {
    fn from(value: oauth2_applications::Model) -> Self {
        Self {
            id: value.id,
            name: value.name,
            secret: value.secret,
            redirect_uri: value.redirect_uri,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
