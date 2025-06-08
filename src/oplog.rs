use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Write},
    sync::{Arc, Mutex},
};
use uuid::Uuid;
use log::{warn};

/// Represents different timer operations that can be logged
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum LogOperation {
    SetTimer {
        timer_id: Uuid,
        expires_at: u64,
        data: String,
    },
    RemoveTimer {
        timer_id: Uuid,
    },
}

/// A log entry containing timestamp and operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogEntry {
    pub timestamp: u64,
    pub operation: LogOperation,
}

/// Persistent operation log for timer operations
#[derive(Debug, Clone)]
pub struct OpLog {
    file: Arc<Mutex<BufWriter<File>>>,
    log_path: String,
}

impl OpLog {
    /// Creates a new operation log at the specified path
    pub fn new(log_path: &str) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;

        Ok(OpLog {
            file: Arc::new(Mutex::new(BufWriter::new(file))),
            log_path: log_path.to_string(),
        })
    }

    /// Appends a log entry to the operation log
    pub fn append_log(&self, entry: LogEntry) -> std::io::Result<()> {
        let mut file = self.file.lock().expect("Failed to lock log file");
        let serialized = serde_json::to_string(&entry)?;
        writeln!(file, "{}", serialized)?;
        file.flush()?;
        drop(file);
        Ok(())
    }

    /// Reads all log entries from the operation log
    pub fn read_logs(&self) -> std::io::Result<Vec<LogEntry>> {
        let file = File::open(&self.log_path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if !line.trim().is_empty() {
                match serde_json::from_str::<LogEntry>(&line) {
                    Ok(entry) => entries.push(entry),
                    Err(e) => warn!("Failed to deserialize log entry: {}", e),
                }
            }
        }

        Ok(entries)
    }
}
