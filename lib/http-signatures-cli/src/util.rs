use owo_colors::{OwoColorize, Stream};
use std::fmt::Display;

#[inline]
pub fn error_kaomoji() -> impl Display {
    "(┬┬﹏┬┬)".if_supports_color(Stream::Stdout, |text| text.red())
}

#[inline]
pub fn success_kaomoji() -> impl Display {
    "(^///^)".if_supports_color(Stream::Stdout, |text| text.green())
}
