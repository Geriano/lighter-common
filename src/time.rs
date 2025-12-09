use chrono::{DateTime, NaiveDateTime, ParseError, Utc};

pub fn now() -> NaiveDateTime {
    Utc::now().naive_local()
}

pub fn unix() -> u64 {
    now().and_utc().timestamp_millis() as u64
}

pub fn from_str<T: AsRef<str>, F: ToString>(
    value: T,
    format: F,
) -> Result<NaiveDateTime, ParseError> {
    NaiveDateTime::parse_from_str(value.as_ref(), &format.to_string())
}

pub fn from_unix(value: i64) -> Option<NaiveDateTime> {
    DateTime::from_timestamp_millis(value).map(|dt| dt.naive_local())
}

pub fn format(value: NaiveDateTime) -> String {
    value.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn matches() {
        let now = now();
        let unix = now.timestamp_millis();
        let from_str = from_str(format(now), "%Y-%m-%d %H:%M:%S");
        let from_unix = from_unix(unix);

        assert!(from_str.is_ok());
        assert!(from_unix.is_some());

        let now = format(now);
        let from_str = format(from_str.unwrap());
        let from_unix = format(from_unix.unwrap());

        assert_eq!(now, from_str);
        assert_eq!(now, from_unix);
        assert_eq!(from_str, from_unix);
    }
}
