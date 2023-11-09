use std::ops::Deref;

mod macros;

#[derive(Clone, Debug)]
pub enum CowBox<'a, T> {
    Borrowed(&'a T),
    Owned(Box<T>),
}

impl<'a, T> CowBox<'a, T> {
    #[inline]
    pub fn borrowed(value: &'a T) -> Self {
        Self::Borrowed(value)
    }

    #[inline]
    pub fn owned(value: T) -> Self {
        Self::Owned(Box::new(value))
    }
}

impl<'a, T> Deref for CowBox<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(borrow) => borrow,
            Self::Owned(ref owned) => owned,
        }
    }
}
