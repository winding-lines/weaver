use chrono::{DateTime, FixedOffset, SecondsFormat, Utc};
use lib_error::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Date(DateTime<FixedOffset>);

impl Date {
    pub fn parse(rfc3339_date: &str) -> Result<Date> {
        let inner = DateTime::parse_from_rfc3339(rfc3339_date).map_err(|_| "date parse error")?;
        Ok(Date(inner))
    }

    pub fn age(&self) -> i64 {
        let utc: DateTime<Utc> = Utc::now();
        (utc - self.0.with_timezone(&Utc)).num_seconds()
    }

    // Serialize in a format compatible with Javascript (only second precision).
    pub fn to_js(&self) -> String {
        self.0.to_rfc3339_opts(SecondsFormat::Secs, true)
    }
}

/// Age in seconds
pub fn age(rfc3339_date: &str) -> Result<i64> {
    let parsed = DateTime::parse_from_rfc3339(rfc3339_date).map_err(|_| "map parse error")?;
    let utc: DateTime<Utc> = Utc::now();
    let duration = utc - parsed.with_timezone(&Utc);
    Ok(duration.num_seconds())
}

pub fn pretty_diff(seconds: i64) -> String {
    if seconds < 60 {
        return format!("{} sec", seconds);
    }
    if seconds < 3600 {
        return format!("{:0} min", seconds / 60);
    }
    if seconds < 3600 * 24 {
        return format!("{:2} hours", seconds / 3600);
    }
    format!("{:1} days", seconds / (3600 * 24))
}

pub fn short_diff(seconds: i64) -> String {
    if seconds < 60 {
        return "now".into();
    }
    if seconds < 3600 {
        return format!("{:0}m", seconds / 60);
    }
    if seconds < 3600 * 24 {
        return format!("{:2}h", seconds / 3600);
    }
    format!("{:1}d", seconds / (3600 * 24))
}

pub fn now() -> String {

    let utc: DateTime<Utc> = Utc::now();
    utc.to_rfc3339()
}
