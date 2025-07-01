use std::io::{Error, ErrorKind};

pub fn from_palm_timestamp(timestamp: u32) -> Result<chrono::NaiveDateTime, Error> {
    let seconds = timestamp as i64;
    let epoch = chrono::NaiveDate::from_ymd_opt(1904, 1, 1)
        .and_then(|t| t.and_hms_opt(0, 0, 0))
        .ok_or(Error::from(ErrorKind::InvalidData))?;
    Ok(epoch + chrono::Duration::seconds(seconds))
}
