use lol_html::{
    errors::{RewritingError, SelectorError},
    html_content::Element,
    ElementContentHandlers, HandlerResult, HtmlRewriter, Selector, Settings,
};
use std::{borrow::Cow, ops::ControlFlow, str::FromStr};
use thiserror::Error;

type Result<T, E = Error> = std::result::Result<T, E>;

/// Ignore any content handler "errors", since we use these errors
/// as our means of communicating control flow
macro_rules! handle_error {
    ($error_expr:expr_2021) => {{
        match { $error_expr } {
            Err(::lol_html::errors::RewritingError::ContentHandlerError(..)) => return Ok(()),
            other => other,
        }
    }};
}

#[derive(Debug, Error)]
#[error("small sacrifice for the lol_html gods")]
struct Sacrifice;

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
        H: FnMut(&Element<'_, '_>) -> ControlFlow<()>,
    {
        #[inline]
        fn handler_assert<F>(uwu: F) -> F
        where
            F: FnMut(&mut Element<'_, '_>) -> HandlerResult,
        {
            uwu
        }

        #[inline]
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
                        if handler(el).is_continue() {
                            Ok(())
                        } else {
                            Err(Box::new(Sacrifice))
                        }
                    })),
                )],
                ..Settings::new()
            },
            sink_assert(|_| {}),
        );

        handle_error!(rewriter.write(input.as_ref()))?;
        handle_error!(rewriter.end())?;

        Ok(())
    }
}
