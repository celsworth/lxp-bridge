// simple wrapper around a u64 so we can support converting to influxdb::Timestamp
// with a preset precision - to the second is more than good enough and should let
// Influx store it more efficiently.

use std::time::SystemTime;

#[derive(Debug)]
pub struct UnixTime(pub u64);

impl UnixTime {
    pub fn now() -> Self {
        Self(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        )
    }
}

impl From<UnixTime> for influxdb::Timestamp {
    fn from(u: UnixTime) -> Self {
        influxdb::Timestamp::Seconds(u.0 as u128)
    }
}
