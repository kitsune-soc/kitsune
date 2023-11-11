use std::ops::Deref;

pub mod sanitize;

mod macros;

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
