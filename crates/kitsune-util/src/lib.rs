use rand::{distributions::Alphanumeric, Rng};
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
    rand::thread_rng()
        .sample_iter(Alphanumeric)
        .map(char::from)
        .take(TOKEN_LENGTH)
        .collect()
}

const _: () = {
    assert!(
        std::mem::size_of::<CowBox<'_, [u8; 1024]>>() <= std::mem::size_of::<usize>() * 2,
        "Size guarantee violated (should have the size of two or less machine words)",
    );
};

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

impl<T> Deref for CowBox<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(borrow) => borrow,
            Self::Boxed(ref owned) => owned,
        }
    }
}
