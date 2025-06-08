use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    thread::{self},
    time::Duration,
};

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
struct TimerShip {
    timers: Arc<Timers>,
    timer_data: Arc<TimerData>,
}

impl TimerShip {
    fn new() -> Self {
        let ts = TimerShip {
            timers: Arc::new(Timers::new()),
            timer_data: Arc::new(TimerData::new()),
        };
        {
            let timers_clone = Arc::clone(&ts.timers);
            let timer_data_clone = Arc::clone(&ts.timer_data);
            println!("TimerShip initialized with empty timers and timer data.");
            thread::spawn(move || {
                let timer_ship = TimerShip {
                    timers: timers_clone,
                    timer_data: timer_data_clone,
                };
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
                        // code to send expire time to timer executor via cross beam workers
                        } else {
                            let sleep_duration = Duration::from_secs(timer.get_time_left(now));
                            println!("Waiting for timer to expire: {:?},", timer);
                            thread::sleep(sleep_duration);
                        }
                    } else {
                        thread::sleep(Duration::from_millis(100));
                        // println!("No timers available.");
                    }
                }
            });
        };

        ts
    }
    fn get_expiring_timer(&self) -> Option<Timer> {
        self.timers.peek_timer()
    }
    fn set_timer(&self, expires_at: u64, data: String) -> u64 {
        let new_timer = Timer::new(expires_at);
        let timer_id = new_timer.id;
        self.timer_data.add_data(timer_id, data);
        self.timers.add_timer(new_timer);

        timer_id
    }
    fn remove_timer(&self, timer_id: u64) -> Option<String> {
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

fn main() {
    // Example usage of TimerShip
    let timer_ship = TimerShip::new();
    loop {
        thread::sleep(Duration::from_secs(5));
        let now = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs();

        let timer_id4 = timer_ship.set_timer(now + 20, "Timer 4".to_string());
        println!("Set timer with ID: {}", timer_id4);
    }
    // Simulate some delay before removing a timer
}
