use futures::{executor::block_on, future};
use post_process::{Element, Html};
use pretty_assertions::assert_eq;
use std::borrow::Cow;

#[test]
fn link_transformation() {
    let text = "@真島@goro.org how are you doing? :friday-night: #龍が如く0";
    let transformed = block_on(post_process::transform(text, |elem| async move {
        let transformed = match elem {
            Element::Emote(emote) => Element::Html(Html {
                tag: Cow::Borrowed("a"),
                attributes: vec![(
                    Cow::Borrowed("href"),
                    Cow::Owned(format!("https://example.com/emote/{}", emote.shortcode)),
                )],
                content: Box::new(Element::Emote(emote)),
            }),
            Element::Hashtag(hashtag) => Element::Html(Html {
                tag: Cow::Borrowed("a"),
                attributes: vec![(
                    Cow::Borrowed("href"),
                    Cow::Owned(format!("https://example.com/hashtag/{}", hashtag.content)),
                )],
                content: Box::new(Element::Hashtag(hashtag)),
            }),
            Element::Mention(mention) => Element::Html(Html {
                tag: Cow::Borrowed("a"),
                attributes: vec![(
                    Cow::Borrowed("href"),
                    Cow::Owned(format!(
                        "https://example.com/mention/{}/{}",
                        mention.username,
                        mention.domain.as_deref().unwrap_or_default()
                    )),
                )],
                content: Box::new(Element::Mention(mention)),
            }),
            elem => elem,
        };

        Ok(transformed)
    }))
    .unwrap();

    insta::assert_snapshot!(transformed);
}

#[test]
fn noop_transformation() {
    let text = "@真島@goro.org how are you doing? :friday-night: #龍が如く0";
    let transformed = block_on(post_process::transform(text, future::ok)).unwrap();

    assert_eq!(text, transformed);
}
