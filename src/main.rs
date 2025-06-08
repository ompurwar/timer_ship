use timer_ship::{TimerShip, TimerCallback};
use std::{io::{self, Write}, thread, time::Duration};
use log::{info, error};
use env_logger;
use uuid::Uuid;

fn print_menu() {
    println!("\n🚢 Timer Ship - Interactive CLI");
    println!("═══════════════════════════════");
    println!("1. Set timer with duration");
    println!("2. List duration format examples");
    println!("3. View active timers (not implemented)");
    println!("4. Exit");
    println!("\nDuration formats:");
    println!("  • Milliseconds: 100ms, 1500ms (integers only)");
    println!("  • Seconds: 1s, 2.5s, 30s");
    println!("  • Minutes: 1m, 1.5m, 45m");
    println!("  • Hours: 1h, 2.5hr, 24hr");
    println!("═══════════════════════════════");
}

fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn show_duration_examples() {
    println!("\n📝 Duration Format Examples:");
    println!("┌─────────────┬─────────────────┬─────────────────────────┐");
    println!("│ Format      │ Example         │ Description             │");
    println!("├─────────────┼─────────────────┼─────────────────────────┤");
    println!("│ Milliseconds│ 100ms, 1500ms  │ Integer values only     │");
    println!("│ Seconds     │ 1s, 2.5s, 30s  │ Float values supported  │");
    println!("│ Minutes     │ 1m, 1.5m, 45m  │ Float values supported  │");
    println!("│ Hours       │ 1h, 2.5hr, 24h │ Float values supported  │");
    println!("└─────────────┴─────────────────┴─────────────────────────┘");
    
    println!("\n💡 Examples:");
    println!("  • '500ms' - 500 milliseconds");
    println!("  • '5s' - 5 seconds");
    println!("  • '2.5m' - 2 minutes and 30 seconds");
    println!("  • '1.5hr' - 1 hour and 30 minutes");
    println!("  • '0.5h' - 30 minutes");
}

fn interactive_mode(timer_ship: &TimerShip) {
    loop {
        print_menu();
        
        let choice = get_user_input("\nEnter your choice (1-4): ");
        
        match choice.as_str() {
            "1" => {
                let duration = get_user_input("Enter duration (e.g., 5s, 1.5m, 2hr): ");
                
                if duration.is_empty() {
                    println!("❌ Duration cannot be empty!");
                    continue;
                }
                
                let message = get_user_input("Enter timer message/description: ");
                let final_message = if message.is_empty() {
                    format!("Timer set for {}", duration)
                } else {
                    message
                };
                
                match timer_ship.set_timer_with_duration(&duration, final_message) {
                    Ok(timer_id) => {
                        println!("✅ Timer set successfully!");
                        println!("   ID: {}", timer_id);
                        println!("   Duration: {}", duration);
                        println!("   The timer will fire and log the message when it expires.");
                    },
                    Err(e) => {
                        println!("❌ Failed to set timer: {}", e);
                        println!("   Please check your duration format.");
                    }
                }
            },
            "2" => {
                show_duration_examples();
            },
            "3" => {
                println!("⚠️  List active timers feature not implemented yet.");
            },
            "4" => {
                println!("👋 Goodbye! Timers will continue running in background...");
                break;
            },
            _ => {
                println!("❌ Invalid choice. Please enter 1-4.");
            }
        }
        
        println!("\nPress Enter to continue...");
        let _ = get_user_input("");
    }
}

fn demo_mode(timer_ship: &TimerShip) {
    info!("🎮 Running in demo mode - setting example timers...");
    
    // Set some example timers
    let examples = vec![
        ("3s", "Demo: Quick 3-second timer"),
        ("5s", "Demo: Session timeout simulation"),
        ("8s", "Demo: Cache expiration test"),
        ("10s", "Demo: Retry mechanism timer"),
        ("1.5m", "Demo: Long running task"),
    ];
    
    for (duration, message) in examples {
        match timer_ship.set_timer_with_duration(duration, message.to_string()) {
            Ok(timer_id) => info!("✅ Set demo timer: {} - {}", duration, timer_id),
            Err(e) => error!("❌ Failed to set demo timer {}: {}", duration, e),
        }
    }
    
    info!("🎯 Demo timers set! Watch for expiration messages...");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger with default level if RUST_LOG is not set
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    println!("🚢 Starting Timer Ship Application");

    // Create a callback function for timer expiration
    let callback: TimerCallback = Box::new(|timer_id: Uuid, data: String| {
        println!("\n🔔 ═══════════════════════════════════════");
        println!("   TIMER EXPIRED!");
        println!("   ID: {}", timer_id);
        println!("   Message: {}", data);
        println!("   Time: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
        println!("   ═══════════════════════════════════════");
        
        // Log to the application log as well
        info!("🔔 Timer expired - ID: {}, Message: {}", timer_id, data);
        
        // Custom logic based on message content
        match data.as_str() {
            s if s.contains("session") || s.contains("Session") => {
                info!("🔐 Session management: Timer expired");
            },
            s if s.contains("cache") || s.contains("Cache") => {
                info!("💾 Cache management: Entry expired");
            },
            s if s.contains("retry") || s.contains("Retry") => {
                info!("🔄 Retry mechanism: Executing retry logic");
            },
            s if s.contains("Demo:") => {
                info!("🎮 Demo timer completed");
            },
            _ => {
                info!("⏰ Generic timer completed");
            }
        }
    });

    // Create TimerShip with callback
    let timer_ship = TimerShip::with_callback("timer_operations.log", Some(callback))?;
    
    // Check command line arguments for mode
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("interactive");
    
    match mode {
        "demo" => {
            demo_mode(&timer_ship);
            
            // Keep the application running to see timer expirations
            println!("⏳ Demo mode active. Press Ctrl+C to exit.");
            loop {
                thread::sleep(Duration::from_secs(1));
            }
        },
        "interactive" | _ => {
            info!("🎮 Starting interactive mode");
            interactive_mode(&timer_ship);
        }
    }
    
    Ok(())
}
