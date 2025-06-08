use core::time;
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    thread::{self},
    time::Duration,
};

#[derive(Debug)]
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
        // In a real application, you might want to use a more robust ID generation strategy
        // Here we just return a static value for simplicity

        static COUNTER: AtomicU64 = AtomicU64::new(1);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }
    fn is_expired(&self, current_time: u64) -> bool {
        current_time >= self.expires_at
    }
    fn get_expires_at(&self) -> u64 {
        self.expires_at
    }
    fn get_time_left(&self, current_time: u64) -> u64 {
        if self.expires_at > current_time {
            self.expires_at - current_time
        } else {
            0
        }
    }
}







 
fn main() {
    let a: Arc<Mutex<Vec<Timer>>> = Arc::new(Mutex::new(Vec::new()));
    {
        let mut local_timer = a.lock().expect("Failed to lock mutex");
        let now = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs();
        let timer = Timer::new(now + 1);
        local_timer.push(timer);
        let timer = Timer::new(now + 2);
        local_timer.push(timer);
        let timer = Timer::new(now + 3);
        local_timer.push(timer);
        local_timer.sort_by(|a, b| a.expires_at.cmp(&b.expires_at));
    }
    println!("Timer List: {:?}", a);

    let handle = {
        let a = Arc::clone(&a);
        thread::spawn(move || {
            loop {
                let mut timers = a.lock().expect("Failed to lock mutex");
                if let Some(timer) = timers.first() {
                    let now = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs ();
                    if timer.is_expired(now) {
                        println!("Timer expired: {:?}", timers.remove(0));
                        drop(timers);
                    } else {
                        let sleep_duration=Duration::from_secs(timer.get_time_left(now));
                        println!("Waiting for timer to expire: {:?},", timer);
                        drop(timers);
                        thread::sleep(sleep_duration);
                    }
                } else {
                    drop(timers);
                    thread::sleep(Duration::from_millis(100));
                    // println!("No timers available.");
                }
            }
        })
    };
    let h = {
        let a = Arc::clone(&a);
        thread::spawn(move || {
            println!("Starting to add random timers in a loop...");
            loop {
                // Generate a random seed based on current time
                let mut seed = std::time::UNIX_EPOCH.elapsed().unwrap().as_nanos() as u64;
                // Simple linear congruential generator (LCG)
                fn simple_rand(seed: &mut u64) -> u64 {
                    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                    *seed
                }
                let random_secs = 5 + (simple_rand(&mut seed) % 5) as u64;
                let now = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs();
                let timer = Timer::new(now + random_secs);

                {
                    let mut local_timer = a.lock().expect("Failed to lock mutex");
                    local_timer.push(timer);
                    local_timer.sort_by(|a, b| a.expires_at.cmp(&b.expires_at));
                    println!(
                        "Timer List after adding random timer in loop: {:?}",
                        local_timer
                    );
                }

                // Sleep for a random interval before adding the next timer
                let sleep_secs = 1 + (simple_rand(&mut seed) % 3) as u64;
                thread::sleep(Duration::from_secs(sleep_secs));
            }
        })
    };
    handle.join().expect("Thread panicked");
    h.join().expect("Thread panicked");
}
