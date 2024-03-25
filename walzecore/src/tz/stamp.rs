use regex::Captures;

use crate::tz;
use crate::tz::Result;
use crate::tz::DATE_REGEX;
use crate::tz::TIME_REGEX;

pub fn parse_tz_date_time<'a>(
    timezone: &'a str,
    dmy: &'a str,
    hms: &'a str,
) -> Result<'a, (chrono_tz::Tz, Captures<'a>, Captures<'a>)> {
    let tz = match timezone.parse::<chrono_tz::Tz>() {
        Ok(tz) => tz,
        Err(_) => return Err(tz::Error::TzParseFail(timezone)),
    };
    let Some(date) = DATE_REGEX.captures(dmy) else {
        return Err(tz::Error::DateParseError(dmy));
    };
    let Some(time) = TIME_REGEX.captures(hms) else {
        return Err(tz::Error::TimeParseFail(hms));
    };

    Ok((tz, date, time))
}
