use crate::traits::{RenderableEmail, RenderedEmail};
use askama::Template;
use kitsune_error::Result;
use mrml::{mjml::Mjml, prelude::render::RenderOptions};
use typed_builder::TypedBuilder;

#[derive(Template, TypedBuilder)]
#[template(escape = "html", path = "verify.mjml")]
pub struct ConfirmAccount<'a> {
    domain: &'a str,
    username: &'a str,
    verify_link: &'a str,
}

impl RenderableEmail for ConfirmAccount<'_> {
    fn render_email(&self) -> Result<RenderedEmail> {
        let rendered_mjml = self.render()?;
        let parsed_mjml = Mjml::parse(rendered_mjml)?;

        let title = parsed_mjml
            .get_title()
            .expect("[Bug] Missing title in MJML template");
        let body = parsed_mjml.render(&RenderOptions::default())?;

        let plain_text = format!(
            "Confirm your account (@{}) on {}: {}",
            self.username, self.domain, self.verify_link
        );

        Ok(RenderedEmail {
            subject: title,
            body,
            plain_text,
        })
    }
}
