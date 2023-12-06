use hex_simd::{AsOut, AsciiCase};
use std::ops::Deref;

#[doc(hidden)]
pub use tokio;

pub mod convert;
pub mod process;
pub mod sanitize;

mod macros;

const TOKEN_LENGTH: usize = 32;

#[inline]
#[must_use]
pub fn generate_secret() -> String {
    let token_data: [u8; TOKEN_LENGTH] = rand::random();
    let mut buf = [0_u8; TOKEN_LENGTH * 2];

    (*hex_simd::encode_as_str(&token_data, buf.as_mut_slice().as_out(), AsciiCase::Lower))
        .to_string()
}

#[derive(Clone, Debug)]
pub enum CowBox<'a, T> {
    Borrowed(&'a T),
    Boxed(Box<T>),
}

impl<'a, T> CowBox<'a, T> {
    #[inline]
    pub fn borrowed(value: &'a T) -> Self {
        Self::Borrowed(value)
    }

    #[inline]
    pub fn boxed(value: T) -> Self {
        Self::Boxed(Box::new(value))
    }
}

impl<'a, T> Deref for CowBox<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(borrow) => borrow,
            Self::Boxed(ref owned) => owned,
        }
    }
}
