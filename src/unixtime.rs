// simple wrapper around a DateTime so we can support converting to
// influxdb::Timestamp with a preset precision - to the second is more
// than good enough and should let Influx store it more efficiently.

use chrono::{DateTime, Utc};
use serde::Serializer;

#[derive(Debug)]
pub struct UnixTime(DateTime<Utc>);

impl UnixTime {
    pub fn now() -> Self {
        Self(Utc::now())
    }

    // default chrono serialization uses RFC3339 with nanosecond precision..
    // a bit overkill for our uses. clamp it to seconds.
    pub fn serialize<S>(u: &UnixTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // previously used to send ISO8601 string. left for reference.
        // serializer.serialize_str(&u.0.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));

        serializer.serialize_i64(u.0.timestamp())
    }
}

impl From<UnixTime> for influxdb::Timestamp {
    fn from(u: UnixTime) -> Self {
        influxdb::Timestamp::Seconds(u.0.timestamp() as u128)
    }
}
