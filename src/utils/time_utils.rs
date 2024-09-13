use chrono::{Utc,DateTime};

pub fn epoch_todate(now_nanos:i64) -> DateTime<Utc> {
    // Convert the nanosecond timestamp back to seconds and nanoseconds
    let timestamp_seconds = now_nanos / 1_000_000_000;
    let timestamp_nanoseconds = (now_nanos % 1_000_000_000) as u32;

    let datetime_utc = DateTime::<Utc>::from_timestamp(timestamp_seconds, timestamp_nanoseconds).unwrap();
    return datetime_utc;
}