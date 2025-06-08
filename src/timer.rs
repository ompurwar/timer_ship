use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

/// Represents a single timer with expiration time and unique ID
#[derive(Debug, Clone)]
pub struct Timer {
    pub expires_at: u64,
    pub id: u64,
}

impl Timer {
    /// Creates a new timer with the given expiration time
    pub fn new(expires_at: u64) -> Self {
        Timer {
            expires_at,
            id: Timer::generate_id(),
        }
    }

    /// Creates a timer with a specific ID (used for recovery)
    pub fn with_id(expires_at: u64, id: u64) -> Self {
        Timer { expires_at, id }
    }

    fn generate_id() -> u64 {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }

    /// Checks if the timer has expired
    pub fn is_expired(&self, current_time: u64) -> bool {
        current_time >= self.expires_at
    }

    /// Gets the time left until expiration
    pub fn get_time_left(&self, current_time: u64) -> u64 {
        if self.expires_at > current_time {
            self.expires_at - current_time
        } else {
            0
        }
    }
}

/// Container for managing multiple timers
#[derive(Debug, Clone)]
pub struct Timers {
    timers: Arc<Mutex<Vec<Timer>>>,
}

impl Timers {
    pub fn new() -> Self {
        Timers {
            timers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_timer(&self, timer: Timer) {
        let mut local_timers = self.timers.lock().expect("Failed to lock mutex");
        local_timers.push(timer);
        local_timers.sort_by(|a, b| a.expires_at.cmp(&b.expires_at));
        drop(local_timers);
    }

    pub fn peek_timer(&self) -> Option<Timer> {
        let local_timers = self.timers.lock().expect("Failed to lock mutex");
        let result = local_timers.first().cloned();
        drop(local_timers);
        result
    }

    pub fn remove_timer(&self, timer_id: u64) {
        let mut local_timers = self.timers.lock().expect("Failed to lock mutex");
        if let Some(pos) = local_timers.iter().position(|x| x.id == timer_id) {
            local_timers.remove(pos);
        }
        drop(local_timers);
    }
}

impl Default for Timers {
    fn default() -> Self {
        Self::new()
    }
}

/// Container for timer-associated data
#[derive(Debug, Clone)]
pub struct TimerData {
    data: Arc<Mutex<HashMap<u64, String>>>,
}

impl TimerData {
    pub fn new() -> Self {
        TimerData {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_data(&self, timer_id: u64, data: String) {
        let mut local_data = self.data.lock().expect("Failed to lock mutex");
        local_data.insert(timer_id, data);
        drop(local_data);
    }

    pub fn remove_data(&self, timer_id: u64) -> Option<String> {
        let mut local_data = self.data.lock().expect("Failed to lock mutex");
        let data = local_data.remove(&timer_id);
        drop(local_data);
        data
    }
}

impl Default for TimerData {
    fn default() -> Self {
        Self::new()
    }
}
