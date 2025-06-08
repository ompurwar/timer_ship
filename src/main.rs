use timer_util::TimerShip;
use std::{thread, time::Duration};

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
