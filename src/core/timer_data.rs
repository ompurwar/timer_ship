use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

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
