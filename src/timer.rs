use std::{
    collections::{BinaryHeap, HashMap},
    cmp::Ordering,
    sync::{
        Arc, Mutex,
    },
};
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

/// Wrapper for Timer to implement reverse ordering for min-heap behavior
#[derive(Debug, Clone)]
struct TimerHeapItem(Timer);

impl PartialEq for TimerHeapItem {
    fn eq(&self, other: &Self) -> bool {
        self.0.expires_at == other.0.expires_at
    }
}

impl Eq for TimerHeapItem {}

impl PartialOrd for TimerHeapItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TimerHeapItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering to make BinaryHeap behave as min-heap
        other.0.expires_at.cmp(&self.0.expires_at)
    }
}

/// Container for managing multiple timers using a min-heap
#[derive(Debug, Clone)]
pub struct Timers {
    timers: Arc<Mutex<BinaryHeap<TimerHeapItem>>>,
}

impl Timers {
    pub fn new() -> Self {
        Timers {
            timers: Arc::new(Mutex::new(BinaryHeap::new())),
        }
    }

    pub fn add_timer(&self, timer: Timer) {
        let mut local_timers = self.timers.lock().expect("Failed to lock mutex");
        local_timers.push(TimerHeapItem(timer));
        drop(local_timers);
    }

    pub fn peek_timer(&self) -> Option<Timer> {
        let local_timers = self.timers.lock().expect("Failed to lock mutex");
        local_timers.peek().map(|item| item.0.clone())
    }

    pub fn remove_timer(&self, timer_id: Uuid) {
        let mut local_timers = self.timers.lock().expect("Failed to lock mutex");
        
        // Convert heap to vector, remove the timer, and rebuild heap
        let mut timers_vec: Vec<TimerHeapItem> = local_timers.drain().collect();
        timers_vec.retain(|item| item.0.id != timer_id);
        
        // Rebuild the heap from the filtered vector
        *local_timers = timers_vec.into_iter().collect();
        drop(local_timers);
    }

    /// Gets all timers (clone of the internal heap as vector)
    pub fn get_all_timers(&self) -> Vec<Timer> {
        let local_timers = self.timers.lock().expect("Failed to lock mutex");
        local_timers.iter().map(|item| item.0.clone()).collect()
    }
    
    /// Gets the count of timers
    pub fn timer_count(&self) -> usize {
        let local_timers = self.timers.lock().expect("Failed to lock mutex");
        local_timers.len()
    }

    /// Pops the next timer to expire (removes and returns it)
    pub fn pop_timer(&self) -> Option<Timer> {
        let mut local_timers = self.timers.lock().expect("Failed to lock mutex");
        local_timers.pop().map(|item| item.0)
    }

    /// Checks if the timer queue is empty
    pub fn is_empty(&self) -> bool {
        let local_timers = self.timers.lock().expect("Failed to lock mutex");
        local_timers.is_empty()
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
    data: Arc<Mutex<HashMap<Uuid, String>>>,
}

impl TimerData {
    pub fn new() -> Self {
        TimerData {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_data(&self, timer_id: Uuid, data: String) {
        let mut local_data = self.data.lock().expect("Failed to lock mutex");
        local_data.insert(timer_id, data);
        drop(local_data);
    }

    pub fn remove_data(&self, timer_id: Uuid) -> Option<String> {
        let mut local_data = self.data.lock().expect("Failed to lock mutex");
        let data = local_data.remove(&timer_id);
        drop(local_data);
        data
    }
    
    /// Gets data for a specific timer ID
    pub fn get_data(&self, timer_id: Uuid) -> Option<String> {
        let local_data = self.data.lock().expect("Failed to lock mutex");
        local_data.get(&timer_id).cloned()
    }
    
    /// Gets the count of data entries
    pub fn data_count(&self) -> usize {
        let local_data = self.data.lock().expect("Failed to lock mutex");
        local_data.len()
    }
}

impl Default for TimerData {
    fn default() -> Self {
        Self::new()
    }
}
