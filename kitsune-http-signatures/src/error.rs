#![allow(missing_docs)]

use crate::{header::SignatureHeaderBuilderError, BoxError};
use http::header::{InvalidHeaderName, InvalidHeaderValue, ToStrError};
use ring::error::Unspecified;
use std::{num::ParseIntError, time::SystemTimeError};
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Base64Decode(#[from] base64::DecodeError),

    #[error("Signature is expired")]
    ExpiredSignature,

    #[error(transparent)]
    GetKey(BoxError),

    #[error("Missing component")]
    MissingComponent,

    #[error("Signature/Authorization header missing")]
    MissingSignatureHeader,

    #[error(transparent)]
    HttpHeaderToStr(#[from] ToStrError),

    #[error(transparent)]
    InvalidHeaderName(#[from] InvalidHeaderName),

    #[error(transparent)]
    InvalidHeaderValue(#[from] InvalidHeaderValue),

    #[error("Invalid signature header")]
    InvalidSignatureHeader,

    #[error(transparent)]
    TaskJoin(#[from] JoinError),

    #[error(transparent)]
    ParseInt(#[from] ParseIntError),

    #[error(transparent)]
    RingUnspecified(#[from] Unspecified),

    #[error(transparent)]
    SignatureHeaderBuilder(#[from] SignatureHeaderBuilderError),

    #[error(transparent)]
    SystemTime(#[from] SystemTimeError),
}
