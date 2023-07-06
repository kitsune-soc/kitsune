use askama::Template;
use mrml::{mjml::Mjml, prelude::render::Options as RenderOptions};
use thiserror::Error;
use typed_builder::TypedBuilder;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Templating(#[from] askama::Error),

    #[error(transparent)]
    RenderParsing(#[from] mrml::prelude::parse::Error),

    #[error(transparent)]
    Rendering(#[from] mrml::prelude::render::Error),
}

#[derive(Template, TypedBuilder)]
#[template(escape = "html", path = "verify.mjml")]
struct VerifyMail<'a> {
    domain: &'a str,
    username: &'a str,
    verify_link: &'a str,
}

impl VerifyMail<'_> {
    fn render_email(&self) -> Result<String> {
        let rendered_mjml = self.render()?;
        let parsed_mjml = Mjml::parse(rendered_mjml)?;

        parsed_mjml
            .render(&RenderOptions::default())
            .map_err(Error::from)
    }
}

#[cfg(test)]
mod test {
    use crate::VerifyMail;
    use insta::assert_snapshot;

    #[test]
    fn test_render() {
        let mail = VerifyMail {
            domain: "citadel-station.example",
            username: "shodan",
            verify_link: "https://citadel-station.example/verify/perfect-immortal-machine",
        };
        assert_snapshot!(mail.render_email().unwrap());
    }
}
