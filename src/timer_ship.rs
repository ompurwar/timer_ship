use crate::{
    duration_parser::{current_time_ms, parse_duration},
    oplog::{LogEntry, LogOperation, OpLog},
    timer::{Timer, TimerData, Timers},
};
use log::{debug, error, info, warn};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self},
    time::Duration,
};
use uuid::Uuid;

/// Callback function type for timer expiration
pub type TimerCallback = Box<dyn Fn(Uuid, String) + Send + Sync>;

/// Information about an active timer for display purposes
#[derive(Debug, Clone)]
pub struct TimerInfo {
    pub id: Uuid,
    pub expires_at: u64,
    pub data: String,
    pub time_left_ms: u64,
}

impl TimerInfo {
    /// Formats the time left in a human-readable format
    pub fn format_time_left(&self) -> String {
        let ms = self.time_left_ms;
        
        if ms == 0 {
            return "Expired".to_string();
        }
        
        let hours = ms / (1000 * 60 * 60);
        let minutes = (ms % (1000 * 60 * 60)) / (1000 * 60);
        let seconds = (ms % (1000 * 60)) / 1000;
        let milliseconds = ms % 1000;
        
        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else if seconds > 0 {
            format!("{}s {}ms", seconds, milliseconds)
        } else {
            format!("{}ms", milliseconds)
        }
    }
    
    /// Formats the expiration time as a relative timestamp
    pub fn format_expires_at(&self) -> String {
        use std::time::UNIX_EPOCH;
        
        let expires_at_secs = self.expires_at / 1000;
        let _expires_at_system = UNIX_EPOCH + std::time::Duration::from_secs(expires_at_secs);
        
        // For simplicity, just show the timestamp
        format!("in {}", self.format_time_left())
    }
}

/// Main timer management system with persistent operation logging
#[derive(Clone)]
pub struct TimerShip {
    timers: Arc<Timers>,
    timer_data: Arc<TimerData>,
    oplog: Arc<OpLog>,
    recovery_complete: Arc<AtomicBool>,
    callback: Option<Arc<TimerCallback>>,
}

impl std::fmt::Debug for TimerShip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TimerShip")
            .field("timers", &self.timers)
            .field("timer_data", &self.timer_data)
            .field("oplog", &self.oplog)
            .field("recovery_complete", &self.recovery_complete)
            .field("has_callback", &self.callback.is_some())
            .finish()
    }
}

impl TimerShip {
    /// Creates a new TimerShip with operation logging
    pub fn new(log_path: &str) -> std::io::Result<Self> {
        Self::with_callback(log_path, None)
    }

    /// Creates a new TimerShip with operation logging and expiration callback
    pub fn with_callback(log_path: &str, callback: Option<TimerCallback>) -> std::io::Result<Self> {
        let oplog = Arc::new(OpLog::new(log_path)?);
        let recovery_complete = Arc::new(AtomicBool::new(false));

        let ts = TimerShip {
            timers: Arc::new(Timers::new()),
            timer_data: Arc::new(TimerData::new()),
            oplog,
            recovery_complete: recovery_complete.clone(),
            callback: callback.map(Arc::new),
        };

        // Recover from logs before starting the timer thread
        ts.recover_from_logs()?;
        recovery_complete.store(true, Ordering::Relaxed);
        info!("Recovery from logs completed.");

        // Start the timer processing thread only after recovery
        {
            let timer_ship = ts.clone();
            thread::spawn(move || {
                // Wait for recovery to complete
                while !timer_ship.recovery_complete.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(10));
                }

                info!("Timer processing thread started.");
                loop {
                    let timer = timer_ship.get_expiring_timer();
                    if let Some(timer) = timer {
                        let now = current_time_ms();
                        if timer.is_expired(now) {
                            let timer_id = timer.id;
                            match timer_ship.remove_timer(timer_id) {
                                Ok(data) => {
                                    info!("Timer expired: ID {} : at: {}", timer_id, now);

                                    // Call the expiration callback if provided
                                    if let (Some(callback), Some(data)) = (&timer_ship.callback, data) {
                                        callback(timer_id, data);
                                    }
                                }
                                Err(e) => error!("Error removing expired timer: {}", e),
                            }
                        } else {
                            let sleep_duration_ms = timer.get_time_left(now);
                            let sleep_duration = Duration::from_millis(sleep_duration_ms);
                            debug!("Waiting for timer to expire: {:?}", timer);
                            thread::sleep(sleep_duration);
                        }
                    } else {
                        thread::sleep(Duration::from_millis(100));
                    }
                }
            });
        }

        Ok(ts)
    }

    /// Recovers timer state from operation logs
    fn recover_from_logs(&self) -> std::io::Result<()> {
        info!("Starting recovery from logs...");
        let logs = self.oplog.read_logs()?;
        let log_count = logs.len();

        for entry in &logs {
            match &entry.operation {
                LogOperation::SetTimer {
                    timer_id,
                    expires_at,
                    data,
                } => {
                    let timer = Timer::with_id(*expires_at, *timer_id);
                    self.timer_data.add_data(*timer_id, data.clone());
                    self.timers.add_timer(timer);
                    debug!("Recovered SetTimer: ID {}, expires_at {}", timer_id, expires_at);
                }
                LogOperation::RemoveTimer { timer_id } => {
                    self.timers.remove_timer(*timer_id);
                    self.timer_data.remove_data(*timer_id);
                    debug!("Recovered RemoveTimer: ID {}", timer_id);
                }
            }
        }

        info!("Recovery completed. Processed {} log entries.", log_count);
        Ok(())
    }

    /// Gets the next timer to expire
    pub fn get_expiring_timer(&self) -> Option<Timer> {
        self.timers.peek_timer()
    }

    /// Sets a new timer with associated data
    pub fn set_timer(&self, expires_at: u64, data: String) -> std::io::Result<Uuid> {
        let new_timer = Timer::new(expires_at);
        let timer_id = new_timer.id;

        // Log the operation first
        let log_entry = LogEntry {
            timestamp: current_time_ms(),
            operation: LogOperation::SetTimer {
                timer_id,
                expires_at,
                data: data.clone(),
            },
        };
        self.oplog.append_log(log_entry)?;

        // Then apply the operation
        self.timer_data.add_data(timer_id, data);
        self.timers.add_timer(new_timer);

        Ok(timer_id)
    }

    /// Sets a new timer with duration string (e.g., "1.5s", "100ms", "2m")
    pub fn set_timer_with_duration(&self, duration_str: &str, data: String) -> Result<Uuid, Box<dyn std::error::Error>> {
        let duration_ms = parse_duration(duration_str)?;
        let expires_at = current_time_ms() + duration_ms;
        Ok(self.set_timer_at(expires_at, data)?)
    }

    /// Sets a new timer with absolute expiration time in milliseconds
    pub fn set_timer_at(&self, expires_at: u64, data: String) -> std::io::Result<Uuid> {
        let new_timer = Timer::new(expires_at);
        let timer_id = new_timer.id;

        // Log the operation first
        let log_entry = LogEntry {
            timestamp: current_time_ms(),
            operation: LogOperation::SetTimer {
                timer_id,
                expires_at,
                data: data.clone(),
            },
        };
        self.oplog.append_log(log_entry)?;

        // Then apply the operation
        self.timer_data.add_data(timer_id, data);
        self.timers.add_timer(new_timer);

        Ok(timer_id)
    }

    /// Removes a timer and returns its associated data
    pub fn remove_timer(&self, timer_id: Uuid) -> std::io::Result<Option<String>> {
        // Log the operation first
        let log_entry = LogEntry {
            timestamp: current_time_ms(),
            operation: LogOperation::RemoveTimer { timer_id },
        };
        self.oplog.append_log(log_entry)?;

        // Then apply the operation
        Ok(self.remove_timer_internal(timer_id))
    }

    fn remove_timer_internal(&self, timer_id: Uuid) -> Option<String> {
        self.timers.remove_timer(timer_id);
        let data = self.timer_data.remove_data(timer_id);
        if let Some(ref data_str) = data {
            debug!("Removed timer data: {}", data_str);
        } else {
            warn!("No data found for timer ID: {}", timer_id);
        }
        data
    }

    /// Lists all active timers with their information
    pub fn list_active_timers(&self) -> Vec<TimerInfo> {
        let current_time = current_time_ms();
        let mut timer_infos = Vec::new();
        
        // Get all timers using the new public method
        let timers = self.timers.get_all_timers();
        
        for timer in timers {
            if let Some(data) = self.get_timer_data(timer.id) {
                let time_left = if timer.expires_at > current_time {
                    timer.expires_at - current_time
                } else {
                    0
                };
                
                timer_infos.push(TimerInfo {
                    id: timer.id,
                    expires_at: timer.expires_at,
                    data,
                    time_left_ms: time_left,
                });
            }
        }
        
        // Sort by expiration time (soonest first)
        timer_infos.sort_by(|a, b| a.expires_at.cmp(&b.expires_at));
        
        timer_infos
    }
    
    /// Gets the data associated with a timer ID
    fn get_timer_data(&self, timer_id: Uuid) -> Option<String> {
        self.timer_data.get_data(timer_id)
    }
    
    /// Gets the count of active timers
    pub fn active_timer_count(&self) -> usize {
        self.timers.timer_count()
    }
}
