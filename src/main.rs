use timer_util::{TimerShip, TimerCallback};
use std::{thread, time::Duration};
use log::{info, error};
use env_logger;
use uuid::Uuid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger with default level if RUST_LOG is not set
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    info!("Starting timer utility application");

    // Create a callback function for timer expiration
    let callback: TimerCallback = Box::new(|timer_id: Uuid, data: String| {
        info!("🔔 Timer callback fired! ID: {}, Data: {}", timer_id, data);
        
        // Here you can add your custom logic:
        // - Send notifications
        // - Execute scheduled tasks
        // - Clean up resources
        // - Trigger other operations
        
        match data.as_str() {
            s if s.contains("session:") => {
                info!("Session expired, cleaning up user session");
            },
            s if s.contains("cache_key:") => {
                info!("Cache entry expired, removing from cache");
            },
            s if s.contains("retry_task:") => {
                info!("Retry timer expired, executing retry logic");
            },
            _ => {
                info!("Generic timer expired: {}", data);
            }
        }
    });

    // Example usage of TimerShip with callback
    let timer_ship = TimerShip::with_callback("timer_operations.log", Some(callback))?;

    // Set some example timers
    let _ = timer_ship.set_timer_with_duration("5s", "session:user123".to_string());
    let _ = timer_ship.set_timer_with_duration("3s", "cache_key:data456".to_string());
    let _ = timer_ship.set_timer_with_duration("8s", "retry_task:job789".to_string());

    loop {
        thread::sleep(Duration::from_secs(10));

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
