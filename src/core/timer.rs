use uuid::Uuid;

/// Represents a single timer with expiration time and unique ID
#[derive(Debug, Clone)]
pub struct Timer {
    pub expires_at: u64, // milliseconds since UNIX epoch
    pub id: Uuid,
}

impl Timer {
    /// Creates a new timer with the given expiration time in milliseconds
    pub fn new(expires_at: u64) -> Self {
        Timer {
            expires_at,
            id: Uuid::new_v4(),
        }
    }

    /// Creates a timer with a specific ID (used for recovery)
    pub fn with_id(expires_at: u64, id: Uuid) -> Self {
        Timer { expires_at, id }
    }

    /// Checks if the timer has expired
    pub fn is_expired(&self, current_time: u64) -> bool {
        current_time >= self.expires_at
    }

    /// Gets the time left until expiration in milliseconds
    pub fn get_time_left(&self, current_time: u64) -> u64 {
        if self.expires_at > current_time {
            self.expires_at - current_time
        } else {
            0
        }
    }
}
