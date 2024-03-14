// 2022-03-04 05:06:07 hardcoded time for tests
#[allow(dead_code)]
const HARDCODED_TEST_TIME: i64 = 1646370367;

pub struct Utils;
impl Utils {
    pub fn round(x: f64, decimals: u32) -> f64 {
        let y = 10i32.pow(decimals) as f64;
        (x * y).round() / y
    }

    pub fn u16ify(array: &[u8], offset: usize) -> u16 {
        u16::from_le_bytes([array[offset], array[offset + 1]])
    }

    #[cfg(not(feature = "mocks"))]
    pub fn utc() -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }

    #[cfg(feature = "mocks")]
    pub fn utc() -> chrono::DateTime<chrono::Utc> {
        use chrono::TimeZone;
        // [22, 3, 4, 5, 6, 7] hardcoded for tests
        chrono::Utc.timestamp(HARDCODED_TEST_TIME, 0)
    }

    #[cfg(not(feature = "mocks"))]
    pub fn localtime() -> chrono::DateTime<chrono::Local> {
        chrono::Local::now()
    }

    #[cfg(feature = "mocks")]
    pub fn localtime() -> chrono::DateTime<chrono::Local> {
        use chrono::TimeZone;
        // [22, 3, 4, 5, 6, 7] hardcoded for tests
        chrono::Local.timestamp(HARDCODED_TEST_TIME, 0)
    }
}
