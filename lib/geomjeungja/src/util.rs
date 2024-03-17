use std::{fmt, ops::Deref};

#[derive(Clone)]
pub struct OpaqueDebug<T>(pub T);

impl<T> Deref for OpaqueDebug<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> fmt::Debug for OpaqueDebug<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(std::any::type_name::<T>())
            .finish_non_exhaustive()
    }
}
