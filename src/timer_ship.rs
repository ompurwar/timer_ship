use crate::{
    duration_parser::{current_time_ms, parse_duration, ParseError},
    oplog::{LogEntry, LogOperation, OpLog},
    timer::{Timer, TimerData, Timers},
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self},
    time::Duration,
};
use uuid::Uuid;

/// Main timer management system with persistent operation logging
#[derive(Debug, Clone)]
pub struct TimerShip {
    timers: Arc<Timers>,
    timer_data: Arc<TimerData>,
    oplog: Arc<OpLog>,
    recovery_complete: Arc<AtomicBool>,
}

impl TimerShip {
    /// Creates a new TimerShip with operation logging
    pub fn new(log_path: &str) -> std::io::Result<Self> {
        let oplog = Arc::new(OpLog::new(log_path)?);
        let recovery_complete = Arc::new(AtomicBool::new(false));

        let ts = TimerShip {
            timers: Arc::new(Timers::new()),
            timer_data: Arc::new(TimerData::new()),
            oplog,
            recovery_complete: recovery_complete.clone(),
        };

        // Recover from logs before starting the timer thread
        ts.recover_from_logs()?;
        recovery_complete.store(true, Ordering::Relaxed);
        println!("Recovery from logs completed.");

        // Start the timer processing thread only after recovery
        {
            let timer_ship = ts.clone();
            thread::spawn(move || {
                // Wait for recovery to complete
                while !timer_ship.recovery_complete.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(10));
                }

                println!("Timer processing thread started.");
                loop {
                    let timer = timer_ship.get_expiring_timer();
                    if let Some(timer) = timer {
                        let now = current_time_ms();
                        if timer.is_expired(now) {
                            match timer_ship.remove_timer(timer.id) {
                                Ok(data) => println!("Timer expired: {:?} : at: {}", data, now),
                                Err(e) => eprintln!("Error removing expired timer: {}", e),
                            }
                        } else {
                            let sleep_duration_ms = timer.get_time_left(now);
                            let sleep_duration = Duration::from_millis(sleep_duration_ms);
                            println!("Waiting for timer to expire: {:?}", timer);
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
        println!("Starting recovery from logs...");
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
                    println!("Recovered SetTimer: ID {}, expires_at {}", timer_id, expires_at);
                }
                LogOperation::RemoveTimer { timer_id } => {
                    self.timers.remove_timer(*timer_id);
                    self.timer_data.remove_data(*timer_id);
                    println!("Recovered RemoveTimer: ID {}", timer_id);
                }
            }
        }

        println!("Recovery completed. Processed {} log entries.", log_count);
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
            println!("Removed timer data: {}", data_str);
        } else {
            println!("No data found for timer ID: {}", timer_id);
        }
        data
    }
}
