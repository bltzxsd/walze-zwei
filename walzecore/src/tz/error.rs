use thiserror::Error;

use std::result;

pub type Result<'e, T> = result::Result<T, Error<'e>>;

// #[derive(Debug, Error)]
// pub enum Error {
//     #[error("invalid namespace \"{0}\" ")]
//     InvalidNamespace(String),
//     #[error("alias \"{0}\" not found")]
//     AliasNotFound(String),
//     #[error("namespace \"{0}\" not found")]
//     NamespaceNotFound(String),
//     #[error("{0}")]
//     Simple(&'static str),
// }

#[derive(Debug, Error)]
pub enum Error<'e> {
    #[error("{0}")]
    ParseFail(#[from] chrono::ParseError),
    #[error("{0}")]
    RoundingError(#[from] chrono::RoundingError),
    #[error("{0}")]
    ParseMonthFail(#[from] chrono::ParseMonthError),
    #[error("{0}")]
    ParseWeekdayFail(#[from] chrono::ParseWeekdayError),
    #[error("{0}")]
    OutofRange(#[from] chrono::OutOfRangeError),
    #[error("failed to parse timezone {0}")]
    TzParseFail(&'e str),
    #[error("could not parse time {0}")]
    TimeParseFail(&'e str),
    #[error("could not parse date {0}")]
    DateParseError(&'e str),
}
