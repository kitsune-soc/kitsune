use core::fmt;
use std::error::Error as StdError;

#[derive(Debug)]
pub struct EnumConversionError(pub i32);

impl fmt::Display for EnumConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Attempted to convert integer to enum. Got invalid value \"{}\"",
            self.0
        )
    }
}

impl StdError for EnumConversionError {}

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
