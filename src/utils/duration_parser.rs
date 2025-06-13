use std::time::{SystemTime, UNIX_EPOCH};

/// Errors that can occur during duration parsing
#[derive(Debug)]
pub enum ParseError {
    InvalidFormat(String),
    InvalidNumber(String),
    UnknownUnit(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidFormat(s) => write!(f, "Invalid duration format: {}", s),
            ParseError::InvalidNumber(s) => write!(f, "Invalid number: {}", s),
            ParseError::UnknownUnit(s) => write!(f, "Unknown time unit: {}", s),
        }
    }
}

impl std::error::Error for ParseError {}

/// Parses a duration string and returns the duration in milliseconds
/// 
/// Supported formats:
/// - "1ms" => 1 millisecond
/// - "10s" => 10 seconds
/// - "1.5m" => 1.5 minutes
/// - "2hr" => 2 hours
/// - "0.5h" => 0.5 hours
pub fn parse_duration(duration_str: &str) -> Result<u64, ParseError> {
    let duration_str = duration_str.trim().to_lowercase();
    
    if duration_str.is_empty() {
        return Err(ParseError::InvalidFormat("Empty duration string".to_string()));
    }

    // Find where the number ends and unit begins
    let mut split_pos = 0;
    for (i, ch) in duration_str.char_indices() {
        if ch.is_alphabetic() {
            split_pos = i;
            break;
        }
    }

    if split_pos == 0 {
        return Err(ParseError::InvalidFormat("No unit specified".to_string()));
    }

    let (number_part, unit_part) = duration_str.split_at(split_pos);
    
    let number: f64 = number_part.parse()
        .map_err(|_| ParseError::InvalidNumber(number_part.to_string()))?;

    if number < 0.0 {
        return Err(ParseError::InvalidNumber("Duration cannot be negative".to_string()));
    }

    let milliseconds = match unit_part {
        "ms" => number,
        "s" => number * 1000.0,
        "m" => number * 60.0 * 1000.0,
        "h" | "hr" => number * 60.0 * 60.0 * 1000.0,
        _ => return Err(ParseError::UnknownUnit(unit_part.to_string())),
    };

    Ok(milliseconds as u64)
}

/// Gets the current time in milliseconds since UNIX epoch
pub fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_milliseconds() {
        assert_eq!(parse_duration("100ms").unwrap(), 100);
        assert_eq!(parse_duration("1ms").unwrap(), 1);
    }

    #[test]
    fn test_parse_seconds() {
        assert_eq!(parse_duration("1s").unwrap(), 1000);
        assert_eq!(parse_duration("2.5s").unwrap(), 2500);
    }

    #[test]
    fn test_parse_minutes() {
        assert_eq!(parse_duration("1m").unwrap(), 60000);
        assert_eq!(parse_duration("1.5m").unwrap(), 90000);
    }

    #[test]
    fn test_parse_hours() {
        assert_eq!(parse_duration("1h").unwrap(), 3600000);
        assert_eq!(parse_duration("1hr").unwrap(), 3600000);
        assert_eq!(parse_duration("1.5hr").unwrap(), 5400000);
    }
}