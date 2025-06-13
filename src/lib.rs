//! Timer Utility Library
//! 
//! A persistent timer system with operation logging for failure recovery.

pub mod core;
pub mod persistence;
pub mod utils;
pub mod timer_ship;

#[cfg(feature = "performance-tests")]
pub mod testing;

// Re-export main types
pub use timer_ship::{TimerShip, TimerCallback, TimerInfo};
pub use core::Timer;
pub use persistence::{LogEntry, LogOperation};
pub use utils::{parse_duration, ParseError};
pub use uuid::Uuid;

/// Result type for timer operations
pub type TimerResult<T> = std::io::Result<T>;

/// Error types for timer operations
#[derive(Debug)]
pub enum TimerError {
    IoError(std::io::Error),
    SerializationError(serde_json::Error),
}

impl From<std::io::Error> for TimerError {
    fn from(err: std::io::Error) -> Self {
        TimerError::IoError(err)
    }
}

impl From<serde_json::Error> for TimerError {
    fn from(err: serde_json::Error) -> Self {
        TimerError::SerializationError(err)
    }
}

impl std::fmt::Display for TimerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimerError::IoError(e) => write!(f, "IO Error: {}", e),
            TimerError::SerializationError(e) => write!(f, "Serialization Error: {}", e),
        }
    }
}

impl std::error::Error for TimerError {}
