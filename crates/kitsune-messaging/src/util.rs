use std::fmt;

pub struct TransparentDebug<T>(pub T);

impl<T> fmt::Debug for TransparentDebug<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {{ ... }}", std::any::type_name::<T>())
    }
}
