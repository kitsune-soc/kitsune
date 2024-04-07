use crate::{Error, ErrorType};

mod sealed {
    pub trait Sealed {}

    impl<T, E> Sealed for Result<T, E> {}
}

pub trait ResultExt<T>: sealed::Sealed {
    fn with_error_type(self, ty: ErrorType) -> Result<T, Error>;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: Into<Error>,
{
    #[inline]
    fn with_error_type(self, ty: ErrorType) -> Result<T, Error> {
        self.map_err(|err| err.into().with_error_type(ty))
    }
}
