use thiserror::Error;

/// Don't ask me why we need this wrapper, I really don't know.
/// I would have preferred to just use a boxed error but here we are.
#[derive(Debug, Error)]
#[error(transparent)]
pub struct ErrorWrapper(pub kitsune_error::BoxError);
