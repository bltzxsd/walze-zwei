pub mod error;
pub mod stamp;

pub use error::Error;
pub use error::Result;

use lazy_static::lazy_static;
use regex::Captures;
use regex::Regex;

lazy_static! {
    /// Regex is built to accept h in 0-24, m in 0-59, s in 0-59
    static ref TIME_REGEX: Regex = Regex::new(
        r"^(([1-9]|0[0-9]|1[0-9]|2[0-3]){0,1})h(([0-9]|0[0-9]|1[0-9]|2[0-9]|3[0-9]|4[0-9]|5[0-9]){0,1})m(([0-9]|0[0-9]|1[0-9]|2[0-9]|3[0-9]|4[0-9]|5[0-9]){0,1})s$"
    ).unwrap();
    static ref DATE_REGEX: Regex = Regex::new(r"([0-9]{1,2})/([0-9]{1,2})/([0-9]{2,4})\b").unwrap();
}

fn extract_group<'g>(captures: &'g Captures<'g>, idx: usize) -> &'g str {
    captures
        .get(idx)
        .map_or("0", |m| if m.is_empty() { "0" } else { m.as_str() })
}

pub fn hms(captures: &Captures) -> (u32, u32, u32) {
    (
        extract_group(captures, 1).parse().unwrap_or_default(),
        extract_group(captures, 3).parse().unwrap_or_default(),
        extract_group(captures, 5).parse().unwrap_or_default(),
    )
}

pub fn dmy(captures: &Captures) -> (u32, u32, i32) {
    let grp = extract_group(captures, 3).parse().unwrap_or_default();
    let year = if grp < 2000 { grp + 2000 } else { grp };
    (
        extract_group(captures, 1).parse().unwrap_or_default(),
        extract_group(captures, 2).parse().unwrap_or_default(),
        year,
    )
}
