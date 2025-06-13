use std::time::{SystemTime, UNIX_EPOCH};

/// Gets the current time in milliseconds since UNIX epoch
pub fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
