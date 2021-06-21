// simple wrapper around a u64 so we can support converting to influxdb::Timestamp
// with a preset precision - to the second is more than good enough and should let
// Influx store it more efficiently.

use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct UnixTime(DateTime<Utc>);

impl UnixTime {
    pub fn now() -> Self {
        Self(Utc::now())
    }
}

impl From<UnixTime> for influxdb::Timestamp {
    fn from(u: UnixTime) -> Self {
        influxdb::Timestamp::Seconds(DateTime::timestamp(&u.0) as u128)
    }
}
