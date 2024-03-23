use thiserror::Error;

use std::result;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid namespace \"{0}\" ")]
    InvalidNamespace(String),
    #[error("alias \"{0}\" not found")]
    AliasNotFound(String),
    #[error("namespace \"{0}\" not found")]
    NamespaceNotFound(String),
    #[error("{0}")]
    Simple(&'static str),
}
