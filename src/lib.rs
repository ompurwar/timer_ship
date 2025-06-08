//! Timer Utility Library
//! 
//! A persistent timer system with operation logging for failure recovery.

pub mod timer;
pub mod oplog;
pub mod timer_ship;

pub use timer_ship::TimerShip;
pub use timer::Timer;
pub use oplog::{LogEntry, LogOperation};
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
