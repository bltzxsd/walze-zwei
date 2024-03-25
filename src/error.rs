use std::error::Error as StdErr;
pub(crate) type Error = Box<dyn StdErr + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
