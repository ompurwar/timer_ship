use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Write},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread::{self},
    time::Duration,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
enum LogOperation {
    SetTimer {
        timer_id: u64,
        expires_at: u64,
        data: String,
    },
    RemoveTimer {
        timer_id: u64,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct LogEntry {
    timestamp: u64,
    operation: LogOperation,
}

#[derive(Debug, Clone)]
struct OpLog {
    file: Arc<Mutex<BufWriter<File>>>,
    log_path: String,
}

impl OpLog {
    fn new(log_path: &str) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;

        Ok(OpLog {
            file: Arc::new(Mutex::new(BufWriter::new(file))),
            log_path: log_path.to_string(),
        })
    }

    fn append_log(&self, entry: LogEntry) -> std::io::Result<()> {
        let mut file = self.file.lock().expect("Failed to lock log file");
        let serialized = serde_json::to_string(&entry)?;
        writeln!(file, "{}", serialized)?;
        file.flush()?;
        drop(file);
        Ok(())
    }

    fn read_logs(&self) -> std::io::Result<Vec<LogEntry>> {
        let file = File::open(&self.log_path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if !line.trim().is_empty() {
                match serde_json::from_str::<LogEntry>(&line) {
                    Ok(entry) => entries.push(entry),
                    Err(e) => eprintln!("Failed to deserialize log entry: {}", e),
                }
            }
        }

        Ok(entries)
    }
}

#[derive(Debug, Clone)]
struct Timer {
    expires_at: u64,
    id: u64,
}

impl Timer {
    fn new(expires_at: u64) -> Self {
        Timer {
            expires_at,
            id: Timer::generate_id(),
        }
    }
    fn generate_id() -> u64 {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }
    fn is_expired(&self, current_time: u64) -> bool {
        current_time >= self.expires_at
    }
    fn get_time_left(&self, current_time: u64) -> u64 {
        if self.expires_at > current_time {
            self.expires_at - current_time
        } else {
            0
        }
    }
}

#[derive(Debug, Clone)]
struct Timers {
    timers: Arc<Mutex<Vec<Timer>>>,
}

impl Timers {
    fn new() -> Self {
        Timers {
            timers: Arc::new(Mutex::new(Vec::new())),
        }
    }
    fn add_timer(&self, timer: Timer) {
        let mut local_timers = self.timers.lock().expect("Failed to lock mutex");
        local_timers.push(timer);
        local_timers.sort_by(|a, b| a.expires_at.cmp(&b.expires_at));
        drop(local_timers); // Explicitly drop the lock
    }
    fn peek_timer(&self) -> Option<Timer> {
        let local_timers = self.timers.lock().expect("Failed to lock mutex");
        let result = local_timers.first().cloned();
        drop(local_timers); // Explicitly drop the lock
        result
    }
    fn remove_timer(&self, timer_id: u64) {
        let mut local_timers = self.timers.lock().expect("Failed to lock mutex");
        if let Some(pos) = local_timers.iter().position(|x| x.id == timer_id) {
            local_timers.remove(pos);
        }
        drop(local_timers); // Explicitly drop the lock
    }
}

#[derive(Debug, Clone)]
struct TimerData {
    data: Arc<Mutex<HashMap<u64, String>>>,
}
impl TimerData {
    fn new() -> Self {
        TimerData {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    fn add_data(&self, timer_id: u64, data: String) {
        let mut local_data = self.data.lock().expect("Failed to lock mutex");
        local_data.insert(timer_id, data);
        drop(local_data); // Explicitly drop the lock to avoid deadlock
    }
    fn remove_data(&self, timer_id: u64) -> Option<String> {
        let mut local_data = self.data.lock().expect("Failed to lock mutex");
        let data = local_data.remove(&timer_id);
        drop(local_data);
        data
    }
}
#[derive(Debug, Clone)]
struct TimerShip {
    timers: Arc<Timers>,
    timer_data: Arc<TimerData>,
    oplog: Arc<OpLog>,
    recovery_complete: Arc<AtomicBool>,
}

impl TimerShip {
    fn new(log_path: &str) -> std::io::Result<Self> {
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
                        let now = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs();
                        if timer.is_expired(now) {
                            println!(
                                "Timer expired: {:?} : at: {}",
                                timer_ship.remove_timer(timer.id),
                                now
                            );
                        } else {
                            let sleep_duration = Duration::from_secs(timer.get_time_left(now));
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
                    let timer = Timer {
                        expires_at: *expires_at,
                        id: *timer_id,
                    };
                    self.timer_data.add_data(*timer_id, data.clone());
                    self.timers.add_timer(timer);
                    println!(
                        "Recovered SetTimer: ID {}, expires_at {}",
                        timer_id, expires_at
                    );
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

    fn get_expiring_timer(&self) -> Option<Timer> {
        self.timers.peek_timer()
    }

    fn set_timer(&self, expires_at: u64, data: String) -> std::io::Result<u64> {
        let new_timer = Timer::new(expires_at);
        let timer_id = new_timer.id;

        // Log the operation first
        let log_entry = LogEntry {
            timestamp: std::time::UNIX_EPOCH.elapsed().unwrap().as_secs(),
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

    fn remove_timer(&self, timer_id: u64) -> std::io::Result<Option<String>> {
        // Log the operation first
        let log_entry = LogEntry {
            timestamp: std::time::UNIX_EPOCH.elapsed().unwrap().as_secs(),
            operation: LogOperation::RemoveTimer { timer_id },
        };
        self.oplog.append_log(log_entry)?;

        // Then apply the operation
        Ok(self.remove_timer_internal(timer_id))
    }

    fn remove_timer_internal(&self, timer_id: u64) -> Option<String> {
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

fn main() -> std::io::Result<()> {
    // Example usage of TimerShip with oplog
    let timer_ship = TimerShip::new("timer_operations.log")?;

    loop {
        thread::sleep(Duration::from_secs(5));
        let now = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs();

        match timer_ship.set_timer(now + 20, "Timer 4".to_string()) {
            Ok(timer_id) => println!("Set timer with ID: {}", timer_id),
            Err(e) => eprintln!("Failed to set timer: {}", e),
        }
    }
}
