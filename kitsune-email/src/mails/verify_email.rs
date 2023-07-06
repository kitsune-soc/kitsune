use crate::{
    error::Result,
    traits::{RenderableEmail, RenderedEmail},
};
use askama::Template;
use mrml::{mjml::Mjml, prelude::render::Options as RenderOptions};
use typed_builder::TypedBuilder;

#[derive(Template, TypedBuilder)]
#[template(escape = "html", path = "verify.mjml")]
pub struct VerifyEmail<'a> {
    domain: &'a str,
    username: &'a str,
    verify_link: &'a str,
}

impl RenderableEmail for VerifyEmail<'_> {
    fn render_email(&self) -> Result<RenderedEmail> {
        let rendered_mjml = self.render()?;
        let parsed_mjml = Mjml::parse(rendered_mjml)?;

        let title = parsed_mjml
            .get_title()
            .expect("[Bug] Missing title in MJML template");
        let body = parsed_mjml.render(&RenderOptions::default())?;

        Ok(RenderedEmail {
            subject: title,
            body,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{mails::verify_email::VerifyEmail, traits::RenderableEmail};
    use insta::assert_debug_snapshot;

    #[test]
    fn test_render() {
        let mail = VerifyEmail {
            domain: "citadel-station.example",
            username: "shodan",
            verify_link: "https://citadel-station.example/verify/perfect-immortal-machine",
        };
        assert_debug_snapshot!(mail.render_email().unwrap());
    }
}
