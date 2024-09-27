use lol_html::{
    errors::{RewritingError, SelectorError},
    html_content::Element,
    ElementContentHandlers, HandlerResult, HtmlRewriter, Selector, Settings,
};
use std::{borrow::Cow, str::FromStr};
use thiserror::Error;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    InvalidSelector(#[from] SelectorError),

    #[error(transparent)]
    RewriteError(#[from] RewritingError),
}

pub struct Scraper {
    element_selector: Selector,
}

impl Scraper {
    pub fn new(selector: &str) -> Result<Self> {
        Ok(Self {
            element_selector: Selector::from_str(selector)?,
        })
    }

    pub fn process<I, H>(&self, input: I, mut handler: H) -> Result<()>
    where
        I: AsRef<[u8]>,
        H: FnMut(&Element<'_, '_>),
    {
        #[inline(always)]
        fn handler_assert<F>(uwu: F) -> F
        where
            F: FnMut(&mut Element<'_, '_>) -> HandlerResult,
        {
            uwu
        }

        #[inline(always)]
        fn sink_assert<F>(uwu: F) -> F
        where
            F: FnMut(&[u8]),
        {
            uwu
        }

        let mut rewriter = HtmlRewriter::new(
            Settings {
                element_content_handlers: vec![(
                    Cow::Borrowed(&self.element_selector),
                    ElementContentHandlers::default().element(handler_assert(|el| {
                        handler(el);
                        Ok(())
                    })),
                )],
                ..Settings::new()
            },
            sink_assert(|_| {}),
        );

        rewriter.write(input.as_ref())?;
        rewriter.end()?;

        Ok(())
    }
}
