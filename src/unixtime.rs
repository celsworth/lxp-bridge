// simple wrapper around a DateTime so we can support converting to
// influxdb::Timestamp with a preset precision - to the second is more
// than good enough and should let Influx store it more efficiently.

use crate::utils::Utils;

use serde::{Serialize, Serializer};

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct UnixTime(pub chrono::DateTime<chrono::Utc>);

impl UnixTime {
    pub fn now() -> Self {
        Self(Utils::utc())
    }
}

// default chrono serialization uses RFC3339 with nanosecond precision..
// a bit overkill for our uses. clamp it to seconds.
impl Serialize for UnixTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(self.0.timestamp())
    }
}
