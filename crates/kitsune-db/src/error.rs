use core::fmt;
use std::error::Error as StdError;

#[derive(Debug)]
pub struct EnumConversionError<T>(pub T);

impl<T> fmt::Display for EnumConversionError<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Attempted to convert integer to enum. Got invalid value \"{}\"",
            self.0
        )
    }
}

impl<T> StdError for EnumConversionError<T> where T: fmt::Debug + fmt::Display {}

#[derive(Debug)]
pub struct IsoCodeConversionError(pub String);

impl fmt::Display for IsoCodeConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Attempted to convert string to ISO code. Got invalid value \"{}\"",
            self.0
        )
    }
}

impl StdError for IsoCodeConversionError {}
