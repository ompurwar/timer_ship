pub mod duration_parser;
pub mod time;

pub use duration_parser::{parse_duration, ParseError, current_time_ms};
pub use time::current_time_ms as time_current_time_ms;
