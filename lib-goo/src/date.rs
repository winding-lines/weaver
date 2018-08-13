
use chrono::prelude::*;
use lib_error::*;


/// Age in seconds
pub fn age(rfc3339_date: &str) -> Result<i64> {
    let parsed = DateTime::parse_from_rfc3339(rfc3339_date).map_err(
        |_| "map parse error"
    )?;
    let utc: DateTime<Utc> = Utc::now();
    let duration = utc - parsed.with_timezone(&Utc);
    Ok(duration.num_seconds())
}

pub fn pretty_diff(seconds: i64) -> String {
    if seconds < 60 {
        return format!("{} sec", seconds);
    }
    if seconds < 3600 {
        return format!("{:0} min", seconds/60);
    }
    if seconds < 3600*24 {
        return format!("{:2} hours", seconds/3600);
    }
    format!("{:1} days", seconds/(3600*24))
}

pub fn now() -> String {

    let utc: DateTime<Utc> = Utc::now();
    utc.to_rfc3339()
}