use timer_util::TimerShip;
use std::{thread, time::Duration};
use log::{info, error};
use env_logger;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger with default level if RUST_LOG is not set
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    info!("Starting timer utility application");

    // Example usage of TimerShip with oplog
    let timer_ship = TimerShip::new("timer_operations.log")?;

    loop {
        thread::sleep(Duration::from_secs(5));

        // Use duration string instead of absolute time
        match timer_ship.set_timer_with_duration("20s", "Timer with 20 seconds".to_string()) {
            Ok(timer_id) => info!("Set timer with ID: {}", timer_id),
            Err(e) => error!("Failed to set timer: {}", e),
        }

        // Example with different durations
        let _ = timer_ship.set_timer_with_duration("1.5m", "Timer with 1.5 minutes".to_string());
        let _ = timer_ship.set_timer_with_duration("500ms", "Timer with 500 milliseconds".to_string());
        let _ = timer_ship.set_timer_with_duration("2hr", "Timer with 2 hours".to_string());
    }
}
