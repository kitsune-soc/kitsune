use thiserror::Error;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    MailingBackend(BoxError),

    #[error(transparent)]
    Templating(#[from] askama::Error),

    #[error(transparent)]
    RenderParsing(#[from] mrml::prelude::parse::Error),

    #[error(transparent)]
    Rendering(#[from] mrml::prelude::render::Error),
}
